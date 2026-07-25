//! Обработчик событий от потока захвата

use std::{ffi::CStr, os::raw::c_void, sync::Arc};

use common::core::{controller::CoreController, event_handlers::EventHandler};
use ffi::bindings::CaptureEvent;
use logger::*;

pub struct CaptureHandler;

pub struct CaptureEventWrapper {
    pub(crate) event: CaptureEvent,
    pub(crate) message: String,
}

const MODULE: &str = "ScreenCapture";

impl EventHandler<CaptureEventWrapper> for CaptureHandler {
    fn handle(wrapper: CaptureEventWrapper, controller: &CoreController) {
        match wrapper.event {
            CaptureEvent::Initializing => {}
            CaptureEvent::Ready => {
                info!("Streaming started");
                debug!("Message: {}", wrapper.message);
                controller.start();
            }
            CaptureEvent::Error => {
                error!("Capture error: {}", wrapper.message);
                controller.shutdown();
            }
            CaptureEvent::Stopped => {
                info!("Streaming stopped");
                debug!("Message: {}", wrapper.message);
                controller.shutdown();
            }
            CaptureEvent::Paused => {
                info!("Streaming paused");
                debug!("Message: {}", wrapper.message);
            }
            CaptureEvent::Connecting => {
                info!("Streaming connecting");
                debug!("Message: {}", wrapper.message);
            }
        }
    }
}

/// Функция, которую будет дёргать pipewire из C
pub(crate) extern "C" fn callback(
    user_data: *mut c_void,
    event: CaptureEvent,
    message: *const std::os::raw::c_char,
) {
    if user_data.is_null() {
        return;
    }

    let arc_ptr = user_data as *const CoreController;
    let controller = std::mem::ManuallyDrop::new(unsafe { Arc::from_raw(arc_ptr) });

    let string_message = if message.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    };

    let wrapper = CaptureEventWrapper {
        event: event,
        message: string_message,
    };

    CaptureHandler::handle(wrapper, &controller);
}
