use std::sync::Arc;

use worth_execution::time::TimeSampler;
use worth_foundational::time::{MonotonicElapsedError, MonotonicInstant};
use worth_signal::facade::ClockTick;

use super::{BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind};

pub(super) struct BridgeSampledClock {
    sampler: Arc<TimeSampler>,
    origin: MonotonicInstant,
}

impl std::fmt::Debug for BridgeSampledClock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeSampledClock")
            .finish_non_exhaustive()
    }
}

impl BridgeSampledClock {
    pub(super) fn bind(sampler: Arc<TimeSampler>) -> Self {
        let origin = sampler.monotonic();
        Self { sampler, origin }
    }

    pub(super) fn coordinate(&self) -> Result<u64, BridgeManagedTemporalDenial> {
        ClockTick::from_monotonic_elapsed(self.origin, self.sampler.monotonic())
            .map(ClockTick::get)
            .map_err(|error| {
                let detail = match error {
                    MonotonicElapsedError::BootEpochMismatch(_) => {
                        "sampled clock rebooted after installation"
                    }
                    MonotonicElapsedError::EarlierIsLater => {
                        "sampled clock regressed past its origin"
                    }
                };
                BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::ForeignClockTimeline,
                    detail,
                )
            })
    }
}
