//! Модуль, описывающий различные обработчики состояний

use crate::core::controller::CoreController;

pub trait EventHandler<Event> {
    fn handle(event: Event, controller: &CoreController);
}