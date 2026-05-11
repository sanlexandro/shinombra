//! Обработчик событий от потока отправки данных

use common::core::{controller::CoreController, event_handlers::EventHandler};
use hardware_output::HardwareEvents;
use logger::*;

pub struct HardwareHandler;

const MODULE: &str = "HardwareOutput";

impl EventHandler<HardwareEvents> for HardwareHandler {
    fn handle(event: HardwareEvents, controller: &CoreController) {
        match event {
            HardwareEvents::InternalError(error) => {
                error!("Internal error: {}", error);
                controller.shutdown();
            }
            HardwareEvents::Disconnected => {
                error!("Device disconnected");
                controller.shutdown();
            }
            HardwareEvents::NoAccess => {
                error!("No access");
                controller.shutdown();
            }
            HardwareEvents::RetryNeeded => {
                info!("Retry needed");
            }
        }
    }
}
