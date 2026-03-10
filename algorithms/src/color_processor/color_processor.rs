//! Алгоритм обработки массива цвета

#[derive(PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct ScreenConfig {
    pub frame_width: usize,  // ширина кадра
    pub frame_height: usize, // высота кадра
}

pub struct CheckerboardConfig {
    pub chunk_width: usize,  // ширина фрагмента
    pub chunk_height: usize, // высота фрагмента

    pub pixel_step: usize, // шаг чтения пикселей строки
    pub row_stride: usize, // шаг чтения строк
}

/// Ф-я обработки одного фрагмента (жёсткий шахматный порядок)
pub fn process_checkerboard_chunk(
    byte_frame: &[u8],
    screen_config: &ScreenConfig,
    alg_config: &CheckerboardConfig,
    chunk_start_index: usize,
    chunk_orientation: Orientation,
) {
    // Определяем реальный размер строки в байтах
    let row_width = screen_config.frame_width * 4;

    // В зависимости от положения кадра определяем его ширину и высоту
    let chunk_width = if chunk_orientation == Orientation::Horizontal {
        alg_config.chunk_width
    } else {
        alg_config.chunk_height
    };

    let chunk_height = if chunk_orientation == Orientation::Horizontal {
        alg_config.chunk_height
    } else {
        alg_config.chunk_width
    };

    // Цикл построчного чтения (читаем каждую `row_stride` строку)
    for row_idx in (0..chunk_height).step_by(alg_config.row_stride) {
        // Вычисляем стартовый индекс строки (стартовый индекс + (ширина экрана
        // row_strideкол-во строк))
        let row_start_index = chunk_start_index + row_width * row_idx;

        // В зависимости от чётности строки начинаем строку либо с самого начала
        // либо со сдвигом на половину шага чтения строки
        let start = if (row_idx / alg_config.row_stride) % 2 == 1 {
            row_start_index + (alg_config.pixel_step / 2) * 4
        } else {
            row_start_index
        };
        let end = row_start_index + chunk_width * 4;

        // fixme добавить логику смещения строк по чётности

        // Делаем срез строки (от стартового индекса строки, до (него + ширина
        // фрагмента))
        let row_bytes = &byte_frame[start..end];
        let pixels = row_bytes.chunks_exact(4).step_by(alg_config.pixel_step);

        // Итерируемся по срезу (читаем каждый `pixel_step` пиксель)
        for pixel in pixels {
            // вычисление доминирующего значения
        }
    }
}

pub fn alg(byte_frame: &[u8], screen_config: &ScreenConfig, alg_config: &CheckerboardConfig) {
    process_checkerboard_chunk(
        byte_frame,
        screen_config,
        alg_config,
        0,
        Orientation::Horizontal,
    );
}
