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

#include "../include/pw-screen-capture.h"

static atomic_uintptr_t g_active_main_loop = 0;

/**
 * @brief Ф-я остановки потока считывания
 */
void screen_capture_stop_thread(void) {
    struct pw_main_loop *active_loop =
        (struct pw_main_loop *)atomic_load(&g_active_main_loop);

    if (!active_loop) {
        return;
    }

    printf("Stopping capture loop...\n");
    pw_main_loop_quit(active_loop);
}

// Контекст сессии захвата экрана через портал
struct portal_data {
    XdpSession *session;    // сессия портала (ДОЛЖНА БЫТЬ ОТКРЫТА!)
    XdpPortal *portal;      // объект портала
    GMainLoop *event_loop;  // вспомогательный цикл для D-Bus
    uint32_t video_node_id; // идентификатор видеоузла PipeWire
    gboolean ready;         // флаг готовности
};

// Данные приложения для захвата видео
struct stream_data {
    struct pw_main_loop *main_loop;     // основной цикл PipeWire
    struct pw_stream *video_stream;     // видеопоток PipeWire
    struct spa_video_info video_format; // информация о формате видео
    int frame_count;                    // счётчик полученных кадров
    struct portal_data *portal_data;    // данные портала (ДОЛЖНЫ ОСТАТЬСЯ!)
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
static void on_process(void *userdata) {
    struct stream_data *stream_data = userdata;
    struct pw_buffer *pipewire_buffer;  // буфер из PipeWire
    struct spa_buffer *spa_buffer_data; // данные буфера SPA

    printf("on_process called\n");

    // Получаем буфер из видеопотока
    if ((pipewire_buffer =
             pw_stream_dequeue_buffer(stream_data->video_stream)) == NULL) {
        pw_log_warn("out of buffers: %m");
        return;
    }

    spa_buffer_data = pipewire_buffer->buffer;
    if (spa_buffer_data->datas[0].data == NULL) {
        printf("empty buffer\n");
        pw_stream_queue_buffer(stream_data->video_stream, pipewire_buffer);
        return;
    }

    printf("got a frame of size %d\n", spa_buffer_data->datas[0].chunk->size);

    // Определяем цвета пикселей
    uint8_t *pixels = spa_buffer_data->datas[0].data;
    if (pixels) {
        uint8_t b = pixels[0];
        uint8_t g = pixels[1];
        uint8_t r = pixels[2];
        printf("  Frame %d: size=%d, top-left R:%d G:%d B:%d\n",
               stream_data->frame_count + 1,
               spa_buffer_data->datas[0].chunk->size, r, g, b);
    }

    // Возвращаем буфер в видеопоток
    pw_stream_queue_buffer(stream_data->video_stream, pipewire_buffer);

    // Увеличиваем счётчик кадров
    stream_data->frame_count++;
}

/**
 * @brief Обработчик события изменения параметров видеопотока
 * Вызывается при изменении формата видео или других параметров потока
 *
 * @param userdata указатель на структуру stream_data с данными приложения
 * @param id идентификатор параметра, который изменился
 * @param param указатель на структуру параметра SPA или NULL
 */
static void on_param_changed(void *userdata, uint32_t id,
                             const struct spa_pod *param) {
    struct stream_data *stream_data = userdata;

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
    if (spa_format_parse(param, &stream_data->video_format.media_type,
                         &stream_data->video_format.media_subtype) < 0)
        return;

    // Проверяем, что это видео в сыром формате
    if (stream_data->video_format.media_type != SPA_MEDIA_TYPE_video ||
        stream_data->video_format.media_subtype != SPA_MEDIA_SUBTYPE_raw)
        return;

    // Парсим детальные параметры видео
    if (spa_format_video_raw_parse(param, &stream_data->video_format.info.raw) <
        0)
        return;

    uint32_t video_format = stream_data->video_format.info.raw.format;
    printf("Negotiated format: %d (%s)\n", video_format,
           spa_debug_type_find_name(spa_type_video_format, video_format));

    printf("got video format:\n");
    printf("  format: %d (%s)\n", stream_data->video_format.info.raw.format,
           spa_debug_type_find_name(spa_type_video_format,
                                    stream_data->video_format.info.raw.format));
    printf("  size: %dx%d\n", stream_data->video_format.info.raw.size.width,
           stream_data->video_format.info.raw.size.height);
    printf("  framerate: %d/%d\n",
           stream_data->video_format.info.raw.framerate.num,
           stream_data->video_format.info.raw.framerate.denom);
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

/**
 * @brief Главная функция приложения
 * Инициализирует портал и PipeWire, создаёт видеопоток и подключается к узлу
 * захвата
 * @param argc количество аргументов командной строки
 * @param argv массив аргументов командной строки
 * @return 0 при успешном завершении, 1 в случае ошибки
 */
int screen_capture_init(void) {
    printf("Start test\n");

    // Инициализируем портал и создаём сессию захвата (сессия остаётся ОТКРЫТОЙ)
    struct portal_data *portal = get_screencast_session();
    if (!portal) {
        fprintf(stderr, "Failed to start screencast session\n");
        return 1;
    }

    if (portal->video_node_id == 0) {
        fprintf(stderr, "Error: No valid PipeWire node_id from portal\n");
        cleanup_portal_data(portal);
        return 1;
    }

    printf("Connecting PipeWire stream to node %u\n", portal->video_node_id);

    // Данные приложения для работы с видеопотоком
    struct stream_data application_data = {0};
    application_data.frame_count = 0;
    application_data.portal_data = portal;
    const struct spa_pod *format_parameters[1]; // массив параметров формата
    uint8_t builder_buffer[1024]; // буфер для построения SPA объектов
    struct spa_pod_builder pod_builder =
        SPA_POD_BUILDER_INIT(builder_buffer, sizeof(builder_buffer));
    struct pw_properties *stream_properties; // свойства видеопотока

    // Инициализируем PipeWire
    pw_init(0, 0);

    // Создаём основной цикл PipeWire
    application_data.main_loop = pw_main_loop_new(NULL);
    atomic_store(&g_active_main_loop, (uintptr_t)application_data.main_loop);

    // Устанавливаем свойства видеопотока
    stream_properties =
        pw_properties_new(PW_KEY_MEDIA_TYPE, "Video", PW_KEY_MEDIA_CATEGORY,
                          "Capture", PW_KEY_MEDIA_ROLE, "Screen", NULL);

    printf("PipeWire properties configured\n");

    // Создаём видеопоток и подключаем обработчики событий
    application_data.video_stream = pw_stream_new_simple(
        pw_main_loop_get_loop(application_data.main_loop), "video-capture",
        stream_properties, &stream_events, &application_data);

    printf("PipeWire stream created\n");

    // Задаём формат видео: принимаем любой сырой видеоформат
    format_parameters[0] = spa_pod_builder_add_object(
        &pod_builder, SPA_TYPE_OBJECT_Format, SPA_PARAM_EnumFormat,
        SPA_FORMAT_mediaType, SPA_POD_Id(SPA_MEDIA_TYPE_video),
        SPA_FORMAT_mediaSubtype, SPA_POD_Id(SPA_MEDIA_SUBTYPE_raw));

    // Подключаем поток к целевому узлу
    // Явно указываем video_node_id с флагом DRIVER (не AUTOCONNECT), иначе
    // будет подключаться камера
    pw_stream_connect(application_data.video_stream, PW_DIRECTION_INPUT,
                      portal->video_node_id,
                      PW_STREAM_FLAG_AUTOCONNECT | PW_STREAM_FLAG_MAP_BUFFERS |
                          PW_STREAM_FLAG_DRIVER,
                      format_parameters, 1);

    // Запускаем основной цикл
    pw_main_loop_run(application_data.main_loop);
    atomic_store(&g_active_main_loop, (uintptr_t)NULL);

    // Освобождаем ресурсы
    pw_stream_destroy(application_data.video_stream);
    pw_main_loop_destroy(application_data.main_loop);

    // Закрываем и очищаем сессию портала только после остановки цикла
    cleanup_portal_data(portal);

    printf("End test\n");

    return 0;
}
