use algorithms::{analytics::registry::*, processing::registry::*};
use hardware_output::registry::*;

#[derive(Debug)]
pub struct Settings {
    pub chunk_processor_type: ChunkProcessorType,
    pub analytics_type: ColorAccumulatorType,
    pub hardware_output_type: HardwareOutputType,
}

// TODO! Комментарии к коду!
