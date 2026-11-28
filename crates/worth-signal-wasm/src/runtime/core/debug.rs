#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
const WASM_DEBUG_LOGS: bool = true;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_DEBUG_EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

impl super::RuntimeCore {
    pub(super) fn perf_elapsed_ms(&self, start: worth_foundational::time::MonotonicInstant) -> f64 {
        self.runtime
            .time_sampler()
            .monotonic()
            .checked_duration_since(start)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn wasm_debug(message: impl AsRef<str>) {
    if WASM_DEBUG_LOGS {
        WASM_DEBUG_EVENTS.with(|events| {
            events.borrow_mut().push(message.as_ref().to_owned());
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn wasm_debug(_message: impl AsRef<str>) {}

#[cfg(target_arch = "wasm32")]
pub(super) fn take_wasm_debug_events() -> Vec<String> {
    WASM_DEBUG_EVENTS.with(|events| {
        let mut borrowed = events.borrow_mut();
        borrowed.drain(..).collect()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn take_wasm_debug_events() -> Vec<String> {
    Vec::new()
}
