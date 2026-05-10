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

// Конфигурации захвата
typedef struct {
    uint32_t screen_width;
    uint32_t screen_height;
    uint32_t video_format;
} capture_config_t;

// Флаги инициализации
typedef struct {
    bool apply_conversion; // Разрешение применения конвертации форматов
    char *token;      // Сам токен
} initializing_data_t;

// Данные портала и состояния захвата
typedef struct capture_context capture_context_t;

// Состояния потока
typedef enum { Init, Ready, Error, Stopped, Reconnecting } capture_event_t;

// Тип обратного вызова
typedef void (*event_callback_t)(void *user_data, int event, const char *msg);

/**
 * @brief Освобождение памяти Arc<>
 */
void release_user_data(void *user_data);

/**
 * @brief Инициализация захвата экрана
 *
 * @param config Указатель на структуру capture_config_t с настройками захвата
 * @param callback Указатель на функцию обратного вызова `(void *, int, const
 * char *)`
 * @param user_data Пользовательские данные для структуры обратного вызова
 * @param init_data Данные для инициализации
 *
 * @return Указатель на структуру capture_context_t с состоянием захвата, или
 * NULL
 */
capture_context_t *screen_capture_init(capture_config_t *config,
                                       event_callback_t callback,
                                       void *user_data,
                                       initializing_data_t init_data);

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
 * @brief Получение текущей конфигурации захвата
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @return Указатель на текущий конфиг захвата
 */
capture_config_t *get_capture_config(capture_context_t *ctx);

/**
 * @brief Получение токена для восстановления сессии
 *
 * @param ctx Указатель на структуру capture_context_t, возвращённую функцией
 * инициализации
 *
 * @return токен
 */
const char *get_restore_token(capture_context_t *ctx);

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
