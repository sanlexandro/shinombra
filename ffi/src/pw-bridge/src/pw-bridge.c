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

// #define DEBUG_FFI

#ifdef DEBUG_FFI
#define LOG_FFI(fmt, ...) fprintf(stderr, fmt, ##__VA_ARGS__)
#else
#define LOG_FFI(fmt, ...) // Ничего не делаем
#endif

// Контекст сессии захвата экрана через портал
struct portal_data {
    XdpSession *session;    // сессия портала (ДОЛЖНА БЫТЬ ОТКРЫТА!)
    XdpPortal *portal;      // объект портала
    GMainLoop *event_loop;  // вспомогательный цикл для D-Bus
    uint32_t video_node_id; // идентификатор видеоузла PipeWire
    gboolean ready;         // флаг готовности
};

// Состояния, в которых может находиться обработчик
typedef enum { Initializing, Working, Waiting } process_state_t;

// Структура для хранения указателя на текущий кадр
struct current_frame_t {
    uint8_t *current_frame;     // Указатель на DMA
    process_state_t on_process; // Флаг занятости читающего (Consumer)
    bool new_frame;             // Флаг наличия нового кадра (Producer)
    pthread_mutex_t lock;       // Мьютекс для синхронизации доступа к кадру
    pthread_cond_t cond; // Условная переменная для уведомления о новом кадре
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
        frame_data;  // указатель на структуру обмена указателем на кадр
    void *user_data; // указатель на спец данные для функции обратного вызова
    void (*event_callback)(
        void *, int, const char *); // указатель на функцию обратного возврата
    capture_event_t current_state;  // текущее состояние потока
};

/**
 * @brief Разблокировка ожидания
 */
static void unlock_wait(void *user_ctx) {
    capture_context_t *ctx = user_ctx;

    pthread_mutex_lock(&ctx->frame_data.lock);
    pthread_cond_signal(&ctx->frame_data.cond);
    pthread_mutex_unlock(&ctx->frame_data.lock);
}

/**
 * @brief Обработчик события смены состояния потока
 *
 * В зависимости от состояния дёргает переданный callback обработчик
 */
static void on_state_changed(void *user_ctx, enum pw_stream_state old,
                             enum pw_stream_state state, const char *error) {
    capture_context_t *ctx = user_ctx;
    LOG_FFI("Stream state changed: %d -> %d", old, state);

    // В зависимости от состояния вызываем обработчик с соответствующим флагом
    switch (state) {
    case PW_STREAM_STATE_ERROR:
        LOG_FFI(" (error: %s)", error ? error : "Unknown error");
        ctx->current_state = Error;
        unlock_wait(user_ctx);
        ctx->event_callback(ctx->user_data, Error,
                            error ? error : "Unknown error");
        break;

    case PW_STREAM_STATE_STREAMING:
        ctx->current_state = Ready;
        ctx->event_callback(ctx->user_data, Ready, "Streaming started");
        break;

    case PW_STREAM_STATE_UNCONNECTED:
        ctx->current_state = Stopped;
        unlock_wait(user_ctx);
        ctx->event_callback(ctx->user_data, Stopped, "Streaming stopped");
        break;

    default:
        if (ctx->current_state == Ready) {
            ctx->current_state = Reconnecting;
            unlock_wait(user_ctx);
            ctx->event_callback(ctx->user_data, Reconnecting,
                                "Streaming reconnecting");
        }
        break;
    }
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

    LOG_FFI("on_process called\n");

    // Получаем буфер из видеопотока
    if ((pipewire_buffer = pw_stream_dequeue_buffer(ctx->video_stream)) ==
        NULL) {
        pw_log_warn("out of buffers");
        return;
    }

    // Проверяем данные внутри буфера
    struct spa_buffer *spa_buffer_data; // данные буфера SPA
    spa_buffer_data = pipewire_buffer->buffer;
    if (spa_buffer_data->datas[0].data == NULL) {
        LOG_FFI("empty buffer\n");
        pw_stream_queue_buffer(ctx->video_stream, pipewire_buffer);
        return;
    }

    LOG_FFI("got a frame of size %d\n", spa_buffer_data->datas[0].chunk->size);

    // Получаем массив пикселей
    uint8_t *pixels = spa_buffer_data->datas[0].data;

    // И сохраняем в контекст
    if (pixels) {
        LOG_FFI("buffer is ok\n");
        // Лочим и проверяем, свободен ли consumer
        pthread_mutex_lock(&ctx->frame_data.lock);

        // Если процесс занят, просто отпускаем буфер
        if (ctx->frame_data.on_process != Waiting) {
            LOG_FFI("frying buffer\n");
            pthread_mutex_unlock(&ctx->frame_data.lock);
            pw_stream_queue_buffer(ctx->video_stream, pipewire_buffer);
            return;
        }

        // Сохраняем указатель и поднимаем флаг
        ctx->frame_data.current_frame = pixels;
        ctx->frame_data.last_pipewire_buffer = pipewire_buffer;
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
 * @param user_ctx указатель на структуру stream_data с данными приложения
 * @param id идентификатор параметра, который изменился
 * @param param указатель на структуру параметра SPA или NULL
 */
static void on_param_changed(void *user_ctx, uint32_t id,
                             const struct spa_pod *param) {
    capture_context_t *ctx = user_ctx;

    LOG_FFI("on_param_changed called with id=%d\n", id);

    if (param == NULL) {
        LOG_FFI("  param is NULL\n");
        return;
    }

    // Обрабатываем доступные форматы
    if (id == SPA_PARAM_EnumFormat) {
        LOG_FFI("  Available format (EnumFormat)\n");
        return;
    }

    // Нас интересуют только параметры формата
    if (id != SPA_PARAM_Format) {
        LOG_FFI("  Ignoring param type %d\n", id);
        return;
    }

    LOG_FFI("  Processing Format param\n");

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

    LOG_FFI("Negotiated format: %d (%s)\n", ctx->video_format.info.raw.format,
            spa_debug_type_find_name(spa_type_video_format,
                                     ctx->video_format.info.raw.format));

    LOG_FFI("got video format:\n");
    LOG_FFI("  format: %d (%s)\n", ctx->video_format.info.raw.format,
            spa_debug_type_find_name(spa_type_video_format,
                                     ctx->video_format.info.raw.format)); // <--
    LOG_FFI("  size: %dx%d\n", ctx->video_format.info.raw.size.width,
            ctx->video_format.info.raw.size.height);
    LOG_FFI("  framerate: %d/%d\n", ctx->video_format.info.raw.framerate.num,
            ctx->video_format.info.raw.framerate.denom);

    // Если получили адрес конфига, сохраняем
    if (ctx->config) {
        ctx->config->screen_height = ctx->video_format.info.raw.size.height;
        ctx->config->screen_width = ctx->video_format.info.raw.size.width;
        ctx->config->video_format = ctx->video_format.info.raw.format;
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
        LOG_FFI("Failed to start screencast session\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    LOG_FFI("Screencast session started successfully\n");

    // Получаем информацию о видеопотоках из сессии портала
    video_streams = xdp_session_get_streams(session);
    if (video_streams == NULL) {
        LOG_FFI("No streams available from portal\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    // Формат потоков: (узел_id, словарь_свойств)
    g_variant_iter_init(&stream_iterator, video_streams);
    if (g_variant_iter_next(&stream_iterator, "(u@a{sv})", &video_node_id,
                            NULL)) {
        portal_data->video_node_id = video_node_id;
        LOG_FFI("Got PipeWire node_id from portal: %u\n", video_node_id);
        portal_data->ready = TRUE;
    } else {
        LOG_FFI("Failed to parse portal streams\n");
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
        LOG_FFI("Failed to create screencast session\n");
        g_main_loop_quit(portal_data->event_loop);
        return;
    }

    LOG_FFI("Screencast session created, starting...\n");

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
 * @param token Токен для восстановления сессии
 *
 * @return указатель на структуру portal_data с сессией и видеоузлом,
 *         или NULL в случае ошибки. Структура должна быть освобождена
 *         функцией cleanup_portal_data() по завершению!
 */
struct portal_data *get_screencast_session(const char *token) {
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
        LOG_FFI("Failed to create portal connection\n");
        g_main_loop_unref(portal_data->event_loop);
        g_free(portal_data);
        return NULL;
    }

    portal_data->portal = portal;

    LOG_FFI("Waiting for user to select screen/window...\n");

    // Создаём сессию захвата экрана с параметрами:
    // - показываем мониторы и окна
    // - встраиваем курсор в поток
    xdp_portal_create_screencast_session(
        portal, XDP_OUTPUT_MONITOR, XDP_SCREENCAST_FLAG_NONE,
        XDP_CURSOR_MODE_EMBEDDED,
        // Если токен есть — PERSISTENT, если нет — TRANSIENT
        token ? XDP_PERSIST_MODE_PERSISTENT : XDP_PERSIST_MODE_TRANSIENT, token,
        NULL, on_create_screencast_response, portal_data);

    // Ждём, пока сессия захвата будет готова
    g_main_loop_run(portal_data->event_loop);

    if (!portal_data->ready) {
        LOG_FFI("Failed to get screencast session\n");
        g_object_unref(portal);
        g_main_loop_unref(portal_data->event_loop);
        g_free(portal_data);
        return NULL;
    }

    LOG_FFI("Portal screencast setup complete, PipeWire node_id=%u\n",
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
capture_context_t *screen_capture_init(capture_config_t *config,
                                       event_callback_t callback,
                                       void *user_data,
                                       initializing_data_t init_data) {
    // Проверяем, что указатель на конфиг не NULL
    if (!config) {
        return NULL;
    }

    // Проверяем, что указатель на функцию не NULL
    if (!callback) {
        return NULL;
    }

    // Проверяем, что указатель на данные не NULL
    if (!user_data) {
        return NULL;
    }

    // Выделяем память под структуру контекста
    capture_context_t *ctx = malloc(sizeof(capture_context_t));
    // Сохраняем адрес конфига
    ctx->config = config;
    // Сохраняем функцию обратного вызова
    ctx->event_callback = callback;
    // Сохраняем данные
    ctx->user_data = user_data;

    // --- ИНИЦИАЛИЗИРУЕМ PIPEWIRE --- //
    const char *token = strdup(init_data.token);
    // Инициализируем портал и создаём сессию захвата (сессия остаётся ОТКРЫТОЙ)
    struct portal_data *portal = get_screencast_session(token);
    free(token);

    if (!portal) {
        LOG_FFI("Failed to start screencast session\n");
        return NULL;
    }

    if (portal->video_node_id == 0) {
        LOG_FFI("Error: No valid PipeWire node_id from portal\n");
        cleanup_portal_data(portal);
        return NULL;
    }

    LOG_FFI("Connecting PipeWire stream to node %u\n", portal->video_node_id);

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

    LOG_FFI("PipeWire properties configured\n");

    // Создаём видеопоток и подключаем обработчики событий
    ctx->video_stream = pw_stream_new_simple(
        pw_main_loop_get_loop(ctx->main_loop), "video-capture",
        stream_properties, &stream_events, ctx);

    LOG_FFI("PipeWire stream created\n");

    // Задаём формат видео:
    if (init_data.apply_conversion) {
        // если есть флаг преобразования, то забираем любой тип
        format_parameters[0] = spa_pod_builder_add_object(
            &pod_builder, SPA_TYPE_OBJECT_Format, SPA_PARAM_EnumFormat,
            SPA_FORMAT_mediaType, SPA_POD_Id(SPA_MEDIA_TYPE_video),
            SPA_FORMAT_mediaSubtype, SPA_POD_Id(SPA_MEDIA_SUBTYPE_raw));

    } else {
        // принимаем только нативный формат
        format_parameters[0] = spa_pod_builder_add_object(
            &pod_builder, SPA_TYPE_OBJECT_Format, SPA_PARAM_EnumFormat,
            SPA_FORMAT_mediaType, SPA_POD_Id(SPA_MEDIA_TYPE_video),
            SPA_FORMAT_mediaSubtype, SPA_POD_Id(SPA_MEDIA_SUBTYPE_raw),
            SPA_FORMAT_VIDEO_format,
            SPA_POD_CHOICE_ENUM_Id(8, SPA_VIDEO_FORMAT_RGBx,
                                   SPA_VIDEO_FORMAT_BGRx, SPA_VIDEO_FORMAT_xRGB,
                                   SPA_VIDEO_FORMAT_xBGR, SPA_VIDEO_FORMAT_RGBA,
                                   SPA_VIDEO_FORMAT_BGRA, SPA_VIDEO_FORMAT_ARGB,
                                   SPA_VIDEO_FORMAT_ABGR));
    }

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
    ctx->frame_data.on_process = Initializing;
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

    // Освобождаем ресурсы в том же потоке, где работал PipeWire loop
    if (ctx->video_stream && ctx->current_state == Ready) {
        // Уничтожаем поток только если всё ок
        // При возникновении ошибок pipewire удаляет поток самостоятельно и
        // блокирует изменения
        pw_stream_disconnect(ctx->video_stream);
        pw_stream_destroy(ctx->video_stream);
    }
    if (ctx->main_loop) {
        pw_main_loop_destroy(ctx->main_loop);
    }
    if (ctx->portal_data) {
        cleanup_portal_data(ctx->portal_data);
    }
    // Освобождаем Arc на стороне Rust
    if (ctx->user_data) {
        release_user_data(ctx->user_data);
    }

    free(ctx);

    LOG_FFI("End test\n");
}

// Остановка захвата и освобождение ресурсов
void screen_capture_stop(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return;
    }

    // Просим loop завершиться
    // фактическая очистка выполняется в `screen_capture_run`
    if (ctx->main_loop) {
        pw_main_loop_quit(ctx->main_loop);
    }
}

// Получение текущей конфигурации
capture_config_t *get_capture_config(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return NULL;
    }

    // Вытягиваем данные из конфига
    pthread_mutex_lock(&ctx->frame_data.lock);
    capture_config_t *ret = ctx->config;
    pthread_mutex_unlock(&ctx->frame_data.lock);

    return ret;
}

// Получение токена для восстановления сессии
const char *get_restore_token(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return NULL;
    }

    pthread_mutex_lock(&ctx->frame_data.lock);

    // Проверяем, что мы в рабочем состояние для избежания ошибок
    if (ctx->current_state != Ready) {
        pthread_mutex_unlock(&ctx->frame_data.lock);
        return NULL;
    }

    // Вытягиваем данные из сессии
    const char *token =
        xdp_session_get_restore_token(ctx->portal_data->session);
    pthread_mutex_unlock(&ctx->frame_data.lock);

    return token;
}

// Получение указателя на DMA
uint8_t *wait_for_frame(capture_context_t *ctx) {
    // Проверяем, что контекст не NULL
    if (!ctx) {
        return NULL;
    }

    pthread_mutex_lock(&ctx->frame_data.lock);

    // Помечаем, что работа окончена
    ctx->frame_data.on_process = Waiting;

    // Ждём пока Producer не завершит работу или когда поток вышел из состояния
    // готовности
    while (!ctx->frame_data.new_frame && ctx->current_state == Ready) {
        pthread_cond_wait(&ctx->frame_data.cond, &ctx->frame_data.lock);
    }

    // Помечаем, что начали работу
    ctx->frame_data.on_process = Working;
    ctx->frame_data.new_frame = false;

    // Если поток прервался отправляем NULL
    if (ctx->current_state != Ready) {
        return NULL;
    }

    // Сохраняем указатель
    uint8_t *ptr = ctx->frame_data.current_frame;

    // Отпускаем mutex
    pthread_mutex_unlock(&ctx->frame_data.lock);

    return ptr;
}

// Освобождение указатель на DMA
void release_frame(capture_context_t *ctx) {
    // Проверяем, что указатель не NULL
    if (!ctx) {
        return;
    }

    // Блокируем mutex
    pthread_mutex_lock(&ctx->frame_data.lock);

    // Если есть указатель на буфер
    if (ctx->frame_data.last_pipewire_buffer) {
        // Возвращаем буфер в видеопоток
        pw_stream_queue_buffer(ctx->video_stream,
                               ctx->frame_data.last_pipewire_buffer);
        ctx->frame_data.last_pipewire_buffer = NULL;
    }

    // Отпускаем
    pthread_mutex_unlock(&ctx->frame_data.lock);
}