use std::sync::atomic::{AtomicBool, Ordering};

pub struct Signal(AtomicBool);

impl Signal {
    pub const fn new() -> Self {
        Signal(AtomicBool::new(false))
    }

    pub fn on(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn off(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub fn is_on(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

pub static SPINNER_PAUSE: Signal = Signal::new();
