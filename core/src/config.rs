use algorithms::{analytics::registry::*, filters::registry::ColorFilterType, processing::registry::*};
use hardware_output::registry::*;

#[derive(Debug)]
pub struct Settings {
    pub chunk_processor_type: ChunkProcessorType,
    pub analytics_type: ColorAnalystType,
    pub filter_chain: Vec<ColorFilterType>,
    pub hardware_output_type: HardwareOutputType,
}

// TODO! Комментарии к коду!
