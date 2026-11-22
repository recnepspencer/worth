//! Scoped full execution for the independent certification reference.
thread_local! {
    static FULL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub fn disabled() -> bool {
    FULL.with(std::cell::Cell::get)
}
pub fn without_reuse<R>(run: impl FnOnce() -> R) -> R {
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
