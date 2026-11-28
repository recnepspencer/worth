//! The civil calendar local-date grant validity is read on.
//!
//! A calendar is part of a Home's trusted time basis, beside its time source:
//! the host chooses it once while opening, and operation callers cannot
//! supply or replace it. It is a pure mapping from the one admission instant
//! to a local date; it never reads the operating system's zone or clock.

use worth_foundational::facade::{CanonicalDate, CanonicalTimestamp};

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
