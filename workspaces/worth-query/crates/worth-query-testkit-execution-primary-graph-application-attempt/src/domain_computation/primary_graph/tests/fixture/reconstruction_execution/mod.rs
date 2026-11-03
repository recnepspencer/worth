//! Runs the reconstruction tests one at a time.
pub fn isolated_request_owner() -> std::sync::MutexGuard<'static, ()> {
    static RUN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    RUN.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
