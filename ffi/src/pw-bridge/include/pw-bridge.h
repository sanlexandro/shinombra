/**
 * @file pw-screen-capture.h
 *
 * @brief Заголовочный файл для захвата экрана с помощью PipeWire и портала
 */

#ifndef PW_SCREEN_CAPTURE_H
#define PW_SCREEN_CAPTURE_H

#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>

// Структура для хранения конфигурации захвата
typedef struct {
    uint32_t screen_width;
    uint32_t screen_height;
    uint32_t video_format;
} capture_config_t;

// Структура для хранения данных портала и состояния захвата
typedef struct capture_context capture_context_t;

// Состояния потока
typedef enum { Ready, Error, Stopped, Reconnecting } capture_event_t;

// Тип обратного вызова
typedef void (*event_callback_t)(int event, const char *msg);

/**
 * @brief Инициализация захвата экрана
 *
 * @param config Указатель на структуру capture_config_t с настройками захвата
 * @param apply_conversion Флаг включения преобразования формата видео на уровне
 * pipewire
 * @param callback Указатель на функцию обратного вызова `(int, const char *)`
 *
 * @return Указатель на структуру capture_context_t с состоянием захвата, или
 * NULL
 */
capture_context_t *screen_capture_init(capture_config_t *config,
                                       void (*callback)(int, const char *),
                                       bool apply_conversion);

/**
 * @brief Запуск процесса захвата экрана
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @note Эта функция блокирует выполнение, пока захват не будет остановлен
 */
void screen_capture_run(capture_context_t *ctx);

/**
 * @brief Остановка процесса захвата экрана и освобождение ресурсов
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @note Эта функция должна быть вызвана после остановки основного цикла, чтобы
 * гарантировать правильное освобождение ресурсов и закрытие сессии портала
 */
void screen_capture_stop(capture_context_t *ctx);

/**
 * @brief Получение указателя на текущий кадр из видеопотока
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @return Указатель на данные кадра (DMA) или NULL в случае ошибки
 *
 * @note Эта функция блокирует выполнение, пока не будет доступен новый кадр для
 * обработки. После получения кадра, он помечается как "в обработке", и функция
 * возвращает указатель на данные. Важно вызвать эту функцию повторно после
 * завершения обработки кадра, чтобы получить следующий кадр.
 */
uint8_t *wait_for_frame(capture_context_t *ctx);

/**
 * @brief Освобождение текущего кадра после обработки
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @note Эта функция должна быть вызвана после завершения обработки кадра, чтобы
 * пометить его как "свободный" и позволить функции wait_for_frame() получать
 * следующий кадр.
 */
void release_frame(capture_context_t *ctx);

#endif
