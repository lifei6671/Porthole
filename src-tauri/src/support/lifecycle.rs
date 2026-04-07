use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone, Default)]
pub struct AppLifecycleState {
    exit_in_progress: Arc<AtomicBool>,
}

impl AppLifecycleState {
    pub fn begin_exit(&self) -> bool {
        !self.exit_in_progress.swap(true, Ordering::SeqCst)
    }

    pub fn is_exit_in_progress(&self) -> bool {
        self.exit_in_progress.load(Ordering::SeqCst)
    }
}
