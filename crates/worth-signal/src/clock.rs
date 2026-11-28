use std::sync::{Arc, OnceLock};

use worth_execution::time::TimeSampler;
use worth_foundational::time::{ClockSourceIdentity, MonotonicInstant};

/// Serving-graph clock for optional elapsed-time telemetry. It never supplies
/// the canonical clock ticks used by the temporal scheduler.
#[derive(Clone)]
pub(crate) struct SignalTelemetryClock(pub(crate) Arc<TimeSampler>);

impl std::fmt::Debug for SignalTelemetryClock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalTelemetryClock")
            .finish_non_exhaustive()
    }
}

impl Default for SignalTelemetryClock {
    fn default() -> Self {
        static CLOCK: OnceLock<SignalTelemetryClock> = OnceLock::new();
        CLOCK
            .get_or_init(|| {
                let source = ClockSourceIdentity::new(0);
                #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
                let adapter = worth_execution::time::SystemClockAdapter::new(source);
                #[cfg(all(target_family = "wasm", target_os = "unknown"))]
                let adapter = browser::BrowserClockAdapter::new(source);
                Self(Arc::new(TimeSampler::new(Arc::new(adapter))))
            })
            .clone()
    }
}

pub(crate) fn elapsed_nanos(sampler: &TimeSampler, start: MonotonicInstant) -> u128 {
    // A reboot invalidates this optional measurement, never the operation.
    sampler
        .monotonic()
        .checked_duration_since(start)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod browser;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use worth_execution::time::ControlledTime;

    #[test]
    fn elapsed_telemetry_uses_the_injected_clock_and_drops_cross_boot_measurements() {
        let time = Arc::new(ControlledTime::at_unix_secs(10));
        let sampler = TimeSampler::new(time.clone());
        let start = sampler.monotonic();
        time.advance(Duration::from_nanos(7));
        time.step_wall_back(Duration::from_secs(1));
        assert_eq!(elapsed_nanos(&sampler, start), 7);
        time.reboot_epoch();
        time.advance(Duration::from_nanos(9));
        assert_eq!(elapsed_nanos(&sampler, start), 0);
    }
}
