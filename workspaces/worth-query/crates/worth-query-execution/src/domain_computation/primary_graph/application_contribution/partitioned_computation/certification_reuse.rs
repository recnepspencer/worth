//! Scoped full execution for the independent certification reference.
thread_local! {
    static FULL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub(super) fn disabled() -> bool {
    FULL.with(std::cell::Cell::get)
}
pub(super) fn without_reuse<R>(run: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            FULL.with(|full| full.set(self.0));
        }
    }
    let restore = Restore(FULL.with(|full| full.replace(true)));
    let result = run();
    drop(restore);
    result
}
