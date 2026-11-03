use std::time::{SystemTime, UNIX_EPOCH};

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::time_source::{WorthQueryRuntimeTimeSource, WorthQuerySystemRuntimeTimeSource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryRuntimeTimeDenial {
    SourceUnavailable,
    BeforeUnixEpoch,
    TimelineValueOverflow,
    DurationNotRepresentable,
}

/// One trusted-time reading taken from the host-published source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeTimeSample {
    timeline: ApplicationCapabilityValidityTimeline,
    value: AspectValue,
}

pub(in crate::domain_computation) struct WorthQueryRuntimeTimeInterval {
    pub(in crate::domain_computation) issued: WorthQueryRuntimeTimeSample,
    pub(in crate::domain_computation) expires: AspectValue,
}

impl WorthQueryRuntimeTimeSample {
    pub const fn timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.timeline
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

pub(crate) struct WorthQueryRuntimeClock {
    source: Box<dyn WorthQueryRuntimeTimeSource>,
}

impl WorthQueryRuntimeClock {
    pub(in crate::domain_computation) fn system() -> Self {
        Self {
            source: Box::new(WorthQuerySystemRuntimeTimeSource),
        }
    }

    pub(in crate::domain_computation) fn from_source(
        source: impl WorthQueryRuntimeTimeSource,
    ) -> Self {
        Self {
            source: Box::new(source),
        }
    }

    pub(in crate::domain_computation) fn sample(
        &self,
        timeline: ApplicationCapabilityValidityTimeline,
    ) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
        let now = self
            .source
            .current_time()
            .map_err(|_| WorthQueryRuntimeTimeDenial::SourceUnavailable)?;
        sample_at(now, timeline)
    }

    pub(in crate::domain_computation) fn sample_interval(
        &self,
        timeline: ApplicationCapabilityValidityTimeline,
        duration: std::time::Duration,
    ) -> Result<WorthQueryRuntimeTimeInterval, WorthQueryRuntimeTimeDenial> {
        let issued = self.sample(timeline)?;
        let units = duration_units(duration, timeline)?;
        let AspectValue::UInt64(issued_units) = issued.value() else {
            return Err(WorthQueryRuntimeTimeDenial::TimelineValueOverflow);
        };
        let expires = issued_units
            .checked_add(units)
            .map(AspectValue::UInt64)
            .ok_or(WorthQueryRuntimeTimeDenial::TimelineValueOverflow)?;
        Ok(WorthQueryRuntimeTimeInterval { issued, expires })
    }
}

impl worth_proof::FreshnessSource for WorthQueryRuntimeClock {
    type Sample = WorthQueryRuntimeTimeSample;
    type Error = WorthQueryRuntimeTimeDenial;

    fn sample(&self) -> Result<Self::Sample, Self::Error> {
        WorthQueryRuntimeClock::sample(
            self,
            ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
        )
    }
}

fn duration_units(
    duration: std::time::Duration,
    timeline: ApplicationCapabilityValidityTimeline,
) -> Result<u64, WorthQueryRuntimeTimeDenial> {
    match timeline {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => {
            if duration.subsec_nanos() != 0 {
                return Err(WorthQueryRuntimeTimeDenial::DurationNotRepresentable);
            }
            Ok(duration.as_secs())
        }
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => {
            if !duration.subsec_nanos().is_multiple_of(1_000_000) {
                return Err(WorthQueryRuntimeTimeDenial::DurationNotRepresentable);
            }
            duration
                .as_millis()
                .try_into()
                .map_err(|_| WorthQueryRuntimeTimeDenial::TimelineValueOverflow)
        }
    }
}

fn sample_at(
    now: SystemTime,
    timeline: ApplicationCapabilityValidityTimeline,
) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
    let elapsed = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| WorthQueryRuntimeTimeDenial::BeforeUnixEpoch)?;
    let value = match timeline {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => elapsed.as_secs(),
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => elapsed
            .as_millis()
            .try_into()
            .map_err(|_| WorthQueryRuntimeTimeDenial::TimelineValueOverflow)?,
    };
    Ok(WorthQueryRuntimeTimeSample {
        timeline,
        value: AspectValue::UInt64(value),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::time::Duration;

    use super::super::WorthQueryRuntimeTimeSourceDenial;
    use super::*;

    struct SequenceTimeSource {
        samples: Mutex<VecDeque<SystemTime>>,
    }

    impl SequenceTimeSource {
        fn new(samples: impl IntoIterator<Item = SystemTime>) -> Self {
            Self {
                samples: Mutex::new(samples.into_iter().collect()),
            }
        }
    }

    impl WorthQueryRuntimeTimeSource for SequenceTimeSource {
        fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
            self.samples
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .pop_front()
                .ok_or(WorthQueryRuntimeTimeSourceDenial::Unavailable)
        }
    }

    #[test]
    fn scripted_clock_retains_exact_installed_timeline_samples() {
        let clock = WorthQueryRuntimeClock::from_source(SequenceTimeSource::new([
            UNIX_EPOCH + Duration::from_millis(12_345),
            UNIX_EPOCH + Duration::from_millis(12_345),
        ]));
        let seconds = clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .unwrap();
        let milliseconds = clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds)
            .unwrap();
        assert_eq!(seconds.value(), &AspectValue::UInt64(12));
        assert_eq!(milliseconds.value(), &AspectValue::UInt64(12_345));
    }

    #[test]
    fn scripted_clock_exhaustion_and_pre_epoch_samples_fail_closed() {
        let before_epoch = UNIX_EPOCH
            .checked_sub(Duration::from_secs(1))
            .expect("the platform supports a pre-epoch test time");
        let clock = WorthQueryRuntimeClock::from_source(SequenceTimeSource::new([before_epoch]));
        assert_eq!(
            clock.sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds),
            Err(WorthQueryRuntimeTimeDenial::BeforeUnixEpoch)
        );
        assert_eq!(
            clock.sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds),
            Err(WorthQueryRuntimeTimeDenial::SourceUnavailable)
        );
    }
}
