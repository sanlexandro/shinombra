//! Обработчик событий от потока отправки данных

use common::core::{controller::CoreController, event_handlers::EventHandler};
use hardware_output::HardwareEvents;

pub struct HardwareHandler;


impl EventHandler<HardwareEvents> for HardwareHandler {
    fn handle(event: HardwareEvents, controller: &CoreController) {
        match event {
            HardwareEvents::InternalError(error) => {
                println!("[ERROR] HardwareOutput: internal error: {}", error);
                controller.shutdown();
            }
            HardwareEvents::Disconnected => {
                println!("[ERROR] HardwareOutput: device disconnected");
                controller.shutdown();
            },
            HardwareEvents::NoAccess => {
                println!("[ERROR] HardwareOutput: no access");
                controller.shutdown();
            }
            HardwareEvents::RetryNeeded => {
                println!("[ERROR] HardwareOutput: retry needed");
                controller.shutdown();
            }
        }
    }
}
