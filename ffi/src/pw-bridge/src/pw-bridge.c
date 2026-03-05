#include <spa/debug/types.h>
#include <spa/param/video/format-utils.h>
#include <spa/param/video/type-info.h>
#include <spa/pod/builder.h>

#include <pipewire/pipewire.h>

#include <gio/gio.h>
#include <libportal/portal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <time.h>

#include <inttypes.h>
#include <stdbool.h>

#include "../include/pw-bridge.h"

// Контекст сессии захвата экрана через портал
struct portal_data {
    XdpSession *session;    // сессия портала (ДОЛЖНА БЫТЬ ОТКРЫТА!)
    XdpPortal *portal;      // объект портала
    GMainLoop *event_loop;  // вспомогательный цикл для D-Bus
    uint32_t video_node_id; // идентификатор видеоузла PipeWire
    gboolean ready;         // флаг готовности
};

// Структура для хранения указателя на текущий кадр
struct current_frame_t {
    uint8_t *current_frame; // Указатель на DMA
    bool on_process;        // Флаг занятости читающего (Consumer)
    bool new_frame;         // Фдаг наличия нового кадра (Producer)
    pthread_mutex_t lock;   // Мьютекс для синхронизации доступа к кадру
    pthread_cond_t cond;    // Условная переменная для уведомления о новом кадре
    struct pw_buffer
        *last_pipewire_buffer; // последний буфер из видеопотока (для
                               // освобождения после обработки)
};

// Контекст захвата экрана и обработки видеопотока
struct capture_context {
    capture_config_t *config; // указатель на структуру конфигурации

    struct pw_main_loop *main_loop;
    struct pw_stream *video_stream;     // видеопоток PipeWire
    struct spa_video_info video_format; // информация о формате видео
    struct portal_data *portal_data;    // данные портала (ДОЛЖНЫ ОСТАТЬСЯ!)

    struct current_frame_t
        frame_data; // указатель на структуру обмена указателем на кадр
};

/**
 * @brief Обработчик события смены состояния потока
 */
static void on_state_changed(void *userdata, enum pw_stream_state old,
                             enum pw_stream_state state, const char *error) {
    printf("Stream state changed: %d -> %d", old, state);
    if (error)
        printf(" (error: %s)", error);
    printf("\n");
}

/**
 * @brief Обработчик события обработки буфера из видеопотока PipeWire
 * Вызывается при поступлении нового буфера данных видео
 *
 * @param userdata указатель на структуру stream_data с данными приложения
 */
static void on_process(void *user_ctx) {
    capture_context_t *ctx = user_ctx; // получили контекст
    struct pw_buffer *pipewire_buffer; // буфер из PipeWire

    printf("on_process called\n");

    // Получаем буфер из видеопотока
    if ((pipewire_buffer = pw_stream_dequeue_buffer(ctx->video_stream)) ==
        NULL) {
        pw_log_warn("out of buffers: %m");
        return;
    }

    // Если процесс занят, то пропускаем кадр
    if (ctx->frame_data.on_process) {
        pw_stream_queue_buffer(ctx->video_stream, pipewire_buffer);
        return;
    }

    // Проверяем данные внутри буфера
    struct spa_buffer *spa_buffer_data; // данные буфера SPA
    spa_buffer_data = pipewire_buffer->buffer;
    if (spa_buffer_data->datas[0].data == NULL) {
        printf("empty buffer\n");
        pw_stream_queue_buffer(ctx->video_stream, pipewire_buffer);
        return;
    }

    printf("got a frame of size %d\n", spa_buffer_data->datas[0].chunk->size);

    // Получаем массив пикселей
    uint8_t *pixels = spa_buffer_data->datas[0].data;

    // И сохраняем в контекст
    if (pixels) {
        // Лочим
        pthread_mutex_lock(&ctx->frame_data.lock);

        // Сохраняем указатель и поднимаем флаг
        ctx->frame_data.current_frame = pixels;
        ctx->frame_data.new_frame = true;

        // Пробуждаем ожидающие потоки
        pthread_cond_signal(&ctx->frame_data.cond);

        // Отпускаем
        pthread_mutex_unlock(&ctx->frame_data.lock);
    }
}

/**
 * @brief Обработчик события изменения параметров видеопотока
 * Вызывается при изменении формата видео или других параметров потока
 *
 * @param userdata указатель на структуру stream_data с данными приложения
 * @param id идентификатор параметра, который изменился
 * @param param указатель на структуру параметра SPA или NULL
 */
static void on_param_changed(void *user_ctx, uint32_t id,
                             const struct spa_pod *param) {
    capture_context_t *ctx = user_ctx;

    printf("on_param_changed called with id=%d\n", id);

    if (param == NULL) {
        printf("  param is NULL\n");
        return;
    }

    // Обрабатываем доступные форматы
    if (id == SPA_PARAM_EnumFormat) {
        printf("  Available format (EnumFormat)\n");
        return;
    }

    // Нас интересуют только параметры формата
    if (id != SPA_PARAM_Format) {
        printf("  Ignoring param type %d\n", id);
        return;
    }

    printf("  Processing Format param\n");

    // Парсим основные параметры формата
    if (spa_format_parse(param, &ctx->video_format.media_type,
                         &ctx->video_format.media_subtype) < 0)
        return;

    // Проверяем, что это видео в сыром формате
    if (ctx->video_format.media_type != SPA_MEDIA_TYPE_video ||
        ctx->video_format.media_subtype != SPA_MEDIA_SUBTYPE_raw)
        return;

    // Парсим детальные параметры видео
    if (spa_format_video_raw_parse(param, &ctx->video_format.info.raw) < 0)
        return;

    uint32_t video_format = ctx->video_format.info.raw.format;
    printf("Negotiated format: %d (%s)\n", video_format,
           spa_debug_type_find_name(spa_type_video_format, video_format));

    printf("got video format:\n");
    printf("  format: %d (%s)\n", ctx->video_format.info.raw.format,
           spa_debug_type_find_name(spa_type_video_format,
                                    ctx->video_format.info.raw.format));
    printf("  size: %dx%d\n", ctx->video_format.info.raw.size.width,
           ctx->video_format.info.raw.size.height);
    printf("  framerate: %d/%d\n", ctx->video_format.info.raw.framerate.num,
           ctx->video_format.info.raw.framerate.denom);

    // Если получили адресс конфига, сохраняем
    if (ctx->config) {
        ctx->config->screen_height = ctx->video_format.info.raw.size.height;
        ctx->config->screen_width = ctx->video_format.info.raw.size.width;
        ctx->config->is_ready = true;
    }
}

// Обработчики событий видеопотока
static const struct pw_stream_events stream_events = {
    PW_VERSION_STREAM_EVENTS,
    .param_changed = on_param_changed, // при изменении параметров
    .process = on_process,             // при поступлении новых данных
    .state_changed = on_state_changed, // при смене состояния потока
};

/**
 * @brief Обработчик завершения запуска сессии захвата экрана
 * Вызывается после успешного запуска сессии захвата и получает идентификатор
 * видеоузла
 *
 * @param source объект XdpPortal, инициировавший операцию
 * @param result результат асинхронной операции
 * @param user_data указатель на структуру portal_data
 */
static void on_session_start_response(GObject *source, GAsyncResult *result,
                                      gpointer user_data) {
    XdpSession *session = XDP_SESSION(source);
    struct portal_data *portal_data = user_data;
    GVariant *video_streams;      // варианты видеопотоков
    GVariantIter stream_iterator; // итератор по потокам
    guint32 video_node_id;        // идентификатор видеоузла

    // Проверяем, успешно ли запустилась сессия
    if (!xdp_session_start_finish(session, result, NULL)) {
        fprintf(stderr, "Failed to start screencast session\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    printf("Screencast session started successfully\n");

    // Получаем информацию о видеопотоках из сессии портала
    video_streams = xdp_session_get_streams(session);
    if (video_streams == NULL) {
        fprintf(stderr, "No streams available from portal\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    // Формат потоков: (узел_id, словарь_свойств)
    g_variant_iter_init(&stream_iterator, video_streams);
    if (g_variant_iter_next(&stream_iterator, "(u@a{sv})", &video_node_id,
                            NULL)) {
        portal_data->video_node_id = video_node_id;
        printf("Got PipeWire node_id from portal: %u\n", video_node_id);
        portal_data->ready = TRUE;
    } else {
        fprintf(stderr, "Failed to parse portal streams\n");
        portal_data->ready = FALSE;
    }

    g_variant_unref(video_streams);
    g_main_loop_quit(portal_data->event_loop);
}

/**
 * @brief Обработчик успешного создания сессии захвата портала
 * Вызывается после создания новой сессии захвата экрана и запускает процесс
 * диалога выбора
 * @param source объект XdpPortal, инициировавший операцию
 * @param result результат асинхронной операции создания сессии
 * @param user_data указатель на структуру portal_data
 */
static void on_create_screencast_response(GObject *source, GAsyncResult *result,
                                          gpointer user_data) {
    XdpPortal *portal = XDP_PORTAL(source);
    struct portal_data *portal_data = user_data;
    XdpSession *session; // сессия захвата

    // Получаем результат создания сессии
    session = xdp_portal_create_screencast_session_finish(portal, result, NULL);
    if (!session) {
        fprintf(stderr, "Failed to create screencast session\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    printf("Screencast session created, starting...\n");

    // ВАЖНО: сохраняем сессию для длительного использования!
    portal_data->session = session;

    // Запускаем сессию (покажется диалог выбора экрана/окна)
    xdp_session_start(session, NULL, NULL, on_session_start_response,
                      user_data);
    // НЕ удаляем session! Она должна остаться открытой!
}

/**
 * @brief Инициализирует портал и создаёт сессию захвата экрана
 * Осуществляет взаимодействие с системным порталом для получения доступа к
 * захвату экрана/окна и возвращает идентификатор видеоузла PipeWire для
 * подключения. ВАЖНО: сессия портала остаётся открытой для длительного
 * использования!
 *
 * @return указатель на структуру portal_data с сессией и видеоузлом,
 *         или NULL в случае ошибки. Структура должна быть освобождена
 *         функцией cleanup_portal_data() по завершению!
 */
struct portal_data *get_screencast_session(void) {
    struct portal_data *portal_data;
    XdpPortal *portal;

    // Выделяем память для структуры
    portal_data = g_new0(struct portal_data, 1);

    // Создаём основной цикл для асинхронных операций D-Bus
    portal_data->event_loop = g_main_loop_new(NULL, FALSE);
    portal_data->video_node_id = 0;
    portal_data->ready = FALSE;

    // Создаём соединение с порталом
    portal = xdp_portal_new();
    if (!portal) {
        fprintf(stderr, "Failed to create portal connection\n");
        g_main_loop_unref(portal_data->event_loop);
        g_free(portal_data);
        return NULL;
    }

    portal_data->portal = portal;

    printf("Waiting for user to select screen/window...\n");

    // Создаём сессию захвата экрана с параметрами:
    // - показываем мониторы и окна
    // - встраиваем курсор в поток
    // - не восстанавливаем из токена
    xdp_portal_create_screencast_session(
        portal, XDP_OUTPUT_MONITOR, XDP_SCREENCAST_FLAG_NONE,
        XDP_CURSOR_MODE_EMBEDDED, XDP_PERSIST_MODE_TRANSIENT, NULL, NULL,
        on_create_screencast_response, portal_data);

    // Ждём, пока сессия захвата будет готова
    g_main_loop_run(portal_data->event_loop);

    if (!portal_data->ready) {
        fprintf(stderr, "Failed to get screencast session\n");
        g_object_unref(portal);
        g_main_loop_unref(portal_data->event_loop);
        g_free(portal_data);
        return NULL;
    }

    printf("Portal screencast setup complete, PipeWire node_id=%u\n",
           portal_data->video_node_id);

    // ВАЖНО: портал, сессия и event_loop остаются ОТКРЫТЫМИ!
    return portal_data;
}

/**
 * @brief Освобождает ресурсы портала по завершению программы
 *
 * @param portal_data указатель на структуру portal_data
 */
static void cleanup_portal_data(struct portal_data *portal_data) {
    if (!portal_data)
        return;

    if (portal_data->session)
        g_object_unref(portal_data->session);
    if (portal_data->portal)
        g_object_unref(portal_data->portal);
    if (portal_data->event_loop)
        g_main_loop_unref(portal_data->event_loop);

    g_free(portal_data);
}

// Инициализация, запуск и остановка захвата экрана
capture_context_t *screen_capture_init(capture_config_t *config) {
    // Проверяем, что указатель на конфиг не NULL
    if (!config) {
        return NULL;
    }

    // Выделяем память под структуру контекста
    capture_context_t *ctx = malloc(sizeof(capture_context_t));
    // Сохраняем адрес конфига
    ctx->config = config;

    // --- ИНИЦИАЛИЗИРУЕМ PIPEWIRE --- //

    // Инициализируем портал и создаём сессию захвата (сессия остаётся ОТКРЫТОЙ)
    struct portal_data *portal = get_screencast_session();
    if (!portal) {
        fprintf(stderr, "Failed to start screencast session\n");
        return NULL;
    }

    if (portal->video_node_id == 0) {
        fprintf(stderr, "Error: No valid PipeWire node_id from portal\n");
        cleanup_portal_data(portal);
        return NULL;
    }

    printf("Connecting PipeWire stream to node %u\n", portal->video_node_id);

    // Данные приложения для работы с видеопотоком
    ctx->portal_data = portal;
    const struct spa_pod *format_parameters[1]; // массив параметров формата
    uint8_t builder_buffer[1024]; // буфер для построения SPA объектов
    struct spa_pod_builder pod_builder =
        SPA_POD_BUILDER_INIT(builder_buffer, sizeof(builder_buffer));
    struct pw_properties *stream_properties; // свойства видеопотока

    // Инициализируем PipeWire
    pw_init(0, 0);

    // Создаём основной цикл PipeWire
    ctx->main_loop = pw_main_loop_new(NULL);

    // Устанавливаем свойства видеопотока
    stream_properties =
        pw_properties_new(PW_KEY_MEDIA_TYPE, "Video", PW_KEY_MEDIA_CATEGORY,
                          "Capture", PW_KEY_MEDIA_ROLE, "Screen", NULL);

    printf("PipeWire properties configured\n");

    // Создаём видеопоток и подключаем обработчики событий
    ctx->video_stream = pw_stream_new_simple(
        pw_main_loop_get_loop(ctx->main_loop), "video-capture",
        stream_properties, &stream_events, ctx);

    printf("PipeWire stream created\n");

    // Задаём формат видео: принимаем любой сырой видеоформат
    format_parameters[0] = spa_pod_builder_add_object(
        &pod_builder, SPA_TYPE_OBJECT_Format, SPA_PARAM_EnumFormat,
        SPA_FORMAT_mediaType, SPA_POD_Id(SPA_MEDIA_TYPE_video),
        SPA_FORMAT_mediaSubtype, SPA_POD_Id(SPA_MEDIA_SUBTYPE_raw));

    // Подключаем поток к целевому узлу
    // Явно указываем video_node_id с флагом DRIVER (не AUTOCONNECT), иначе
    // будет подключаться камера
    pw_stream_connect(ctx->video_stream, PW_DIRECTION_INPUT,
                      portal->video_node_id,
                      PW_STREAM_FLAG_AUTOCONNECT | PW_STREAM_FLAG_MAP_BUFFERS |
                          PW_STREAM_FLAG_DRIVER,
                      format_parameters, 1);

    // --- ИНИЦИАЛИЗИРУЕМ СТРУКТУРУ ПЕРЕДАЧИ КАДРА --- //

    ctx->frame_data.current_frame = NULL;
    ctx->frame_data.on_process = true;
    ctx->frame_data.new_frame = false;
    pthread_mutex_init(&ctx->frame_data.lock, NULL);
    pthread_cond_init(&ctx->frame_data.cond, NULL);
    ctx->frame_data.last_pipewire_buffer = NULL;

    return ctx;
}

// Запуск основного цикла для обработки видеопотока
void screen_capture_run(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return;
    }

    // Запускаем основной цикл
    pw_main_loop_run(ctx->main_loop);
}

// Остановка захвата и освобождение ресурсов
void screen_capture_stop(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return;
    }

    // Освобождаем ресурсы
    pw_stream_destroy(ctx->video_stream);
    pw_main_loop_destroy(ctx->main_loop);

    // Закрываем и очищаем сессию портала только после остановки цикла
    cleanup_portal_data(ctx->portal_data);

    free(ctx);

    printf("End test\n");
}

// Ф-я получения указателя на DMA
uint8_t *wait_for_frame(capture_context_t *ctx) {
    // Проверяем, что указатель на контекст не равен NULL
    if (ctx == NULL) {
        return NULL;
    }

    // Ждём пока Producer не завершит работу
    while (!ctx->frame_data.new_frame) {
        pthread_cond_wait(&ctx->frame_data.cond, &ctx->frame_data.lock);
    }

    // Помечаем, что начали работу
    ctx->frame_data.on_process = true;
    ctx->frame_data.new_frame = false;

    // Сохраняем указатель
    uint8_t *ptr = ctx->frame_data.current_frame;

    // Отпускаем mutex
    pthread_mutex_unlock(&ctx->frame_data.lock);

    return ptr;
}

// Ф-я отпускающая указатель на DMA
void release_frame(capture_context_t *ctx) {
    // Проверяем, что указатель на контекст не равен NULL
    if (ctx == NULL) {
        return;
    }

    // Блокируем mutex
    pthread_mutex_lock(&ctx->frame_data.lock);

    // Помечаем, что работа окончена
    ctx->frame_data.on_process = false;

    // Если есть указатель на буфер
    if (ctx->frame_data.last_pipewire_buffer) {
        // Возвращаем буфер в видеопоток
        pw_stream_queue_buffer(ctx->video_stream, ctx->frame_data.last_pipewire_buffer);
        ctx->frame_data.last_pipewire_buffer = NULL;
    }

    // Отпускаем
    pthread_mutex_unlock(&ctx->frame_data.lock);
}