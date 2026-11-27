use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use worth_foundational::facade::{AspectValue, CanonicalTimestamp};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::capability_calendar::WorthQueryCapabilityCalendar;
use super::time_source::{WorthQueryRuntimeTimeSource, WorthQuerySystemRuntimeTimeSource};
pub use super::validity_ordinal::validity_ordinal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeTimeDenial {
    SourceUnavailable,
    BeforeUnixEpoch,
    TimelineValueOverflow,
    DurationNotRepresentable,
    /// The Home has no capability calendar for a local-date timeline.
    CalendarUnavailable,
    /// The capability calendar could not name the instant's local date.
    LocalDateUnavailable,
}

impl std::fmt::Display for WorthQueryRuntimeTimeDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::SourceUnavailable => "the trusted runtime time source is unavailable",
            Self::BeforeUnixEpoch => "the trusted time lies before the Unix epoch",
            Self::TimelineValueOverflow => "the trusted time is not representable",
            Self::DurationNotRepresentable => "the duration is not representable on the timeline",
            Self::CalendarUnavailable => "the Home has no capability calendar",
            Self::LocalDateUnavailable => "the capability calendar cannot date the instant",
        })
    }
}

impl std::error::Error for WorthQueryRuntimeTimeDenial {}

/// One trusted-time reading taken from the host-published source: the
/// admission instant and its value on one validity timeline. An admission
/// judges validity and records `admitted_at` from this one reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeTimeSample {
    timeline: ApplicationCapabilityValidityTimeline,
    value: AspectValue,
    instant: CanonicalTimestamp,
}

pub struct WorthQueryRuntimeTimeInterval {
    pub issued: WorthQueryRuntimeTimeSample,
    pub expires: AspectValue,
}

impl WorthQueryRuntimeTimeSample {
    pub const fn timeline(&self) -> ApplicationCapabilityValidityTimeline {
        self.timeline
    }

    pub const fn value(&self) -> &AspectValue {
        &self.value
    }

    /// The trusted instant this sample was read at.
    pub const fn instant(&self) -> CanonicalTimestamp {
        self.instant
    }

    /// This sample as an order-preserving durable ordinal on its timeline.
    pub fn ordinal(&self) -> Option<u64> {
        validity_ordinal(self.timeline, &self.value)
    }
}

/// One trusted admission instant read from `source`, before a runtime clock
/// owns it: a Home stamps its initial-state seed with this reading.
pub fn admission_timestamp_from(
    source: &dyn WorthQueryRuntimeTimeSource,
) -> Result<worth_foundational::facade::CanonicalTimestamp, WorthQueryRuntimeTimeDenial> {
    let now = source
        .current_time()
        .map_err(|_| WorthQueryRuntimeTimeDenial::SourceUnavailable)?;
    super::canonical_instant::timestamp_of(now)
}

pub struct WorthQueryRuntimeClock {
    pub admission_clock:
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    source: Box<dyn WorthQueryRuntimeTimeSource>,
    calendar: Option<Arc<dyn WorthQueryCapabilityCalendar>>,
}

impl WorthQueryRuntimeClock {
    pub fn system() -> Self {
        Self::from_source(WorthQuerySystemRuntimeTimeSource)
    }

    pub fn from_source(source: impl WorthQueryRuntimeTimeSource) -> Self {
        Self {
            admission_clock: source.admission_clock(),
            calendar: source.capability_calendar(),
            source: Box::new(source),
        }
    }

    /// Whether local-date grant validity can be judged on this clock.
    pub fn has_capability_calendar(&self) -> bool {
        self.calendar.is_some()
    }

    /// The trusted current instant as a canonical microsecond UTC timestamp,
    /// read once from the host-published source. Instants before the Unix
    /// epoch are negative; an unrepresentable instant is refused, never clamped.
    pub fn admission_timestamp(
        &self,
    ) -> Result<worth_foundational::facade::CanonicalTimestamp, WorthQueryRuntimeTimeDenial> {
        admission_timestamp_from(self.source.as_ref())
    }

    pub fn sample(
        &self,
        timeline: ApplicationCapabilityValidityTimeline,
    ) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
        self.read()?.sample(timeline)
    }

    /// One reading of the host-published source, to sample on several
    /// validity timelines at the same trusted instant.
    pub fn read(&self) -> Result<WorthQueryRuntimeTimeReading<'_>, WorthQueryRuntimeTimeDenial> {
        let now = self
            .source
            .current_time()
            .map_err(|_| WorthQueryRuntimeTimeDenial::SourceUnavailable)?;
        Ok(WorthQueryRuntimeTimeReading {
            now,
            calendar: self.calendar.as_deref(),
        })
    }

    pub fn sample_interval(
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

/// One trusted instant read once from the host-published source. Every
/// sample taken from it, on any timeline, judges validity at that instant.
pub struct WorthQueryRuntimeTimeReading<'clock> {
    now: SystemTime,
    calendar: Option<&'clock dyn WorthQueryCapabilityCalendar>,
}

impl WorthQueryRuntimeTimeReading<'_> {
    /// This reading's value on `timeline`.
    pub fn sample(
        &self,
        timeline: ApplicationCapabilityValidityTimeline,
    ) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
        sample_at(self.now, timeline, self.calendar)
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
        // Installation refuses governed elevation on these timelines.
        ApplicationCapabilityValidityTimeline::UtcTimestamp
        | ApplicationCapabilityValidityTimeline::LocalDate => {
            Err(WorthQueryRuntimeTimeDenial::DurationNotRepresentable)
        }
    }
}

fn sample_at(
    now: SystemTime,
    timeline: ApplicationCapabilityValidityTimeline,
    calendar: Option<&dyn WorthQueryCapabilityCalendar>,
) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
    let instant = super::canonical_instant::timestamp_of(now)?;
    let value = match timeline {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => {
            AspectValue::UInt64(since_unix_epoch(now)?.as_secs())
        }
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => AspectValue::UInt64(
            since_unix_epoch(now)?
                .as_millis()
                .try_into()
                .map_err(|_| WorthQueryRuntimeTimeDenial::TimelineValueOverflow)?,
        ),
        ApplicationCapabilityValidityTimeline::UtcTimestamp => AspectValue::Timestamp(instant),
        ApplicationCapabilityValidityTimeline::LocalDate => AspectValue::Date(
            calendar
                .ok_or(WorthQueryRuntimeTimeDenial::CalendarUnavailable)?
                .local_date(instant)
                .map_err(|_| WorthQueryRuntimeTimeDenial::LocalDateUnavailable)?,
        ),
    };
    Ok(WorthQueryRuntimeTimeSample {
        timeline,
        value,
        instant,
    })
}

/// Epoch-count timelines cannot express an instant before 1970.
fn since_unix_epoch(now: SystemTime) -> Result<std::time::Duration, WorthQueryRuntimeTimeDenial> {
    now.duration_since(UNIX_EPOCH)
        .map_err(|_| WorthQueryRuntimeTimeDenial::BeforeUnixEpoch)
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
        fn admission_clock(
            &self,
        ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock
        {
            worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(
            )
        }
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

#[cfg(test)]
#[path = "time_basis_timeline_tests.rs"]
mod timeline_tests;

#[path = "time_basis_court_doors_for_test.rs"]
mod court_doors_for_test;
