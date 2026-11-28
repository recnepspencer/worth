use std::sync::Arc;
use worth_execution::time::{RawClockAdapter, TimeSampler};
use worth_foundational::time::AttestedTime;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;

use worth_foundational::facade::{AspectValue, CanonicalTimestamp};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::capability_calendar::WorthQueryCapabilityCalendar;
pub use super::validity_ordinal::validity_ordinal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeTimeDenial {
    SourceUnavailable,
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

/// The trusted wall sampler, request clock and civil calendar installed by a Home.
#[derive(Clone)]
pub struct WorthQueryRuntimeClock {
    admission_clock: WorthQueryAdmissionClock,
    sampler: Arc<TimeSampler>,
    calendar: Option<Arc<dyn WorthQueryCapabilityCalendar>>,
}

impl WorthQueryRuntimeClock {
    pub fn system() -> Self {
        Self::new(worth_execution::time::system_time_sampler().clone())
    }

    pub fn new(sampler: Arc<TimeSampler>) -> Self {
        Self {
            admission_clock: WorthQueryAdmissionClock::system(),
            sampler,
            calendar: None,
        }
    }

    pub fn with_admission_clock(mut self, clock: WorthQueryAdmissionClock) -> Self {
        self.admission_clock = clock;
        self
    }

    pub fn admission_clock(&self) -> &WorthQueryAdmissionClock {
        &self.admission_clock
    }

    pub fn with_capability_calendar(
        mut self,
        calendar: Arc<dyn WorthQueryCapabilityCalendar>,
    ) -> Self {
        self.calendar = Some(calendar);
        self
    }

    pub fn capability_calendar(&self) -> Option<&Arc<dyn WorthQueryCapabilityCalendar>> {
        self.calendar.as_ref()
    }

    /// Whether local-date grant validity can be judged on this clock.
    pub fn has_capability_calendar(&self) -> bool {
        self.calendar.is_some()
    }

    /// The trusted current instant as a canonical microsecond UTC timestamp,
    /// read once from the host-published sampler.
    pub fn admission_timestamp(
        &self,
    ) -> Result<worth_foundational::facade::CanonicalTimestamp, WorthQueryRuntimeTimeDenial> {
        Ok(timestamp_of(self.wall()?))
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
        Ok(WorthQueryRuntimeTimeReading {
            now: self.wall()?,
            calendar: self.calendar.as_deref(),
        })
    }

    fn wall(&self) -> Result<AttestedTime, WorthQueryRuntimeTimeDenial> {
        self.sampler
            .wall()
            .map_err(|_| WorthQueryRuntimeTimeDenial::SourceUnavailable)
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

impl<Adapter: RawClockAdapter> From<Arc<Adapter>> for WorthQueryRuntimeClock {
    fn from(adapter: Arc<Adapter>) -> Self {
        Self::new(Arc::new(TimeSampler::new(adapter)))
    }
}

/// One trusted instant read once from the host-published source. Every
/// sample taken from it, on any timeline, judges validity at that instant.
pub struct WorthQueryRuntimeTimeReading<'clock> {
    now: AttestedTime,
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
    now: AttestedTime,
    timeline: ApplicationCapabilityValidityTimeline,
    calendar: Option<&dyn WorthQueryCapabilityCalendar>,
) -> Result<WorthQueryRuntimeTimeSample, WorthQueryRuntimeTimeDenial> {
    let instant = timestamp_of(now);
    let value = match timeline {
        ApplicationCapabilityValidityTimeline::UnixEpochSeconds => {
            AspectValue::UInt64(now.unix_nanos() / 1_000_000_000)
        }
        ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => {
            AspectValue::UInt64(now.unix_nanos() / 1_000_000)
        }
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

fn timestamp_of(now: AttestedTime) -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: (now.unix_nanos() / 1_000) as i64,
    }
}

#[cfg(test)]
#[path = "time_basis_timeline_tests.rs"]
mod timeline_tests;
