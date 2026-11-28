use std::sync::atomic::{AtomicU64, Ordering};

use wasm_bindgen::{JsCast, JsValue};
use worth_execution::time::RawClockAdapter;
use worth_foundational::time::{BootEpoch, ClockSourceIdentity, TimeUnavailable};

/// Web composition adapter. Performance time has no wall-clock fallback.
pub(super) struct BrowserClockAdapter {
    source: ClockSourceIdentity,
    boot: BootEpoch,
}

impl BrowserClockAdapter {
    pub(super) fn new(source: ClockSourceIdentity) -> Self {
        // Keep browser identities disjoint from Execution's process-local
        // allocator, which starts at one and grows upwards.
        static NEXT_BOOT: AtomicU64 = AtomicU64::new(u64::MAX);
        Self {
            source,
            boot: BootEpoch::new(NEXT_BOOT.fetch_sub(1, Ordering::Relaxed)),
        }
    }
}

impl RawClockAdapter for BrowserClockAdapter {
    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        nanos(js_sys::Date::now()).ok_or(TimeUnavailable::WallOutOfRange)
    }

    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        let ticks = performance_now().and_then(nanos).unwrap_or_default();
        (ticks, self.boot)
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.source
    }
}

fn performance_now() -> Option<f64> {
    let performance =
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("performance")).ok()?;
    let now = js_sys::Reflect::get(&performance, &JsValue::from_str("now")).ok()?;
    now.dyn_ref::<js_sys::Function>()?
        .call0(&performance)
        .ok()?
        .as_f64()
}

fn nanos(milliseconds: f64) -> Option<u64> {
    let value = (milliseconds * 1_000_000.0).round();
    (value >= 0.0 && value < u64::MAX as f64).then_some(value as u64)
}
