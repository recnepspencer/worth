//! The civil calendar local-date grant validity is read on.
//!
//! A calendar is part of a Home's trusted time basis, beside its time source:
//! the host chooses it once while opening, and operation callers cannot
//! supply or replace it. It is a pure mapping from the one admission instant
//! to a local date; it never reads the operating system's zone or clock.
use std::sync::Arc;
use std::time::SystemTime;

use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

use super::time_source::{WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial};

/// The calendar could not name the local date of an instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityCalendarDenial;

/// Maps a trusted UTC instant to the local civil date it falls on.
pub trait WorthQueryCapabilityCalendar: Send + Sync + 'static {
    fn local_date(
        &self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial>;
}

impl<Calendar> WorthQueryCapabilityCalendar for Calendar
where
    Calendar: Fn(CanonicalTimestamp) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial>
        + Send
        + Sync
        + 'static,
{
    fn local_date(
        &self,
        instant: CanonicalTimestamp,
    ) -> Result<CanonicalDate, WorthQueryCapabilityCalendarDenial> {
        self(instant)
    }
}

/// A time source that also carries the Home's capability calendar.
pub struct WorthQueryCalendaredRuntimeTimeSource<Source> {
    source: Source,
    calendar: Arc<dyn WorthQueryCapabilityCalendar>,
}

impl<Source: WorthQueryRuntimeTimeSource> WorthQueryCalendaredRuntimeTimeSource<Source> {
    pub fn new(source: Source, calendar: Arc<dyn WorthQueryCapabilityCalendar>) -> Self {
        Self { source, calendar }
    }
}

impl<Source: WorthQueryRuntimeTimeSource> WorthQueryRuntimeTimeSource
    for WorthQueryCalendaredRuntimeTimeSource<Source>
{
    fn admission_clock(
        &self,
    ) -> worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        self.source.admission_clock()
    }

    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        self.source.current_time()
    }

    fn capability_calendar(&self) -> Option<Arc<dyn WorthQueryCapabilityCalendar>> {
        Some(Arc::clone(&self.calendar))
    }
}
