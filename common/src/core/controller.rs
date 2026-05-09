//! Контроллер ядра

use std::sync::{atomic::{AtomicBool, Ordering}};

pub struct CoreController {
    pub(super) keep_running: AtomicBool,
}

impl CoreController {
    pub fn new(val: bool) -> Self {
        Self {
            keep_running: AtomicBool::new(val),
        }
    }

    pub fn wait(&self) -> bool {
        self.keep_running.load(Ordering::Relaxed)
    }

    pub fn keep_running(&self) -> bool {
        self.keep_running.load(Ordering::Relaxed)
    }

    pub fn start(&self) {
        self.keep_running.store(true, Ordering::SeqCst);
    }

    pub fn shutdown(&self) {
        self.keep_running.store(false, Ordering::SeqCst);
    }
}

