#ifndef PW_SCREEN_CAPTURE_H
#define PW_SCREEN_CAPTURE_H

// Структура обмена настройками
typedef struct {
    uint32_t screen_width;
    uint32_t screen_height;
    bool is_ready;
} CaptureConfig;

typedef struct {
    CaptureConfig *config;
    struct pw_main_loop *main_loop;
    struct pw_stream *video_stream;     // видеопоток PipeWire
    struct spa_video_info video_format; // информация о формате видео
    int frame_count;                    // счётчик полученных кадров
    struct portal_data *portal_data;    // данные портала (ДОЛЖНЫ ОСТАТЬСЯ!)
} CaptureContext;

CaptureContext *screen_capture_init(CaptureConfig *config);
void screen_capture_run(CaptureContext *ctx);
void screen_capture_stop(CaptureContext *ctx);

#endif
