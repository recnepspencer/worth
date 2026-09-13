thread_local! {
    static AFTER_INSTALL: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        std::cell::RefCell::new(None);
    static AFTER_COMPONENT_PROGRESS: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        std::cell::RefCell::new(None);
}

pub(super) fn run_after_install() {
    AFTER_INSTALL.with(|hook| {
        if let Some(hook) = hook.borrow().as_ref() {
            hook();
        }
    });
}

pub(super) fn run_after_component_progress() {
    AFTER_COMPONENT_PROGRESS.with(|hook| {
        if let Some(hook) = hook.borrow().as_ref() {
            hook();
        }
    });
}

pub(super) fn replace_after_install_hook(hook: Option<Box<dyn Fn()>>) {
    AFTER_INSTALL.with(|installed| *installed.borrow_mut() = hook);
}

pub(super) fn replace_after_component_progress_hook(hook: Option<Box<dyn Fn()>>) {
    AFTER_COMPONENT_PROGRESS.with(|installed| *installed.borrow_mut() = hook);
}
