//! The local civil calendar day for desktop applications, without a UTC
//! fallback: a fixed [`WorthQueryUtcOffset`], or a [`WorthQueryTimeZone`]
//! parsed from TZif bytes or loaded from the operating system.
//!
//! ```
//! use worth_query_host::facade::local_calendar::{WorthQueryTimeZone, WorthQueryUtcOffset};
//! use worth_query_host::facade::declaration::application_schema::{
//!     CanonicalDate, CanonicalTimestamp,
//! };
//!
//! // 01:00Z on 2026-11-25 is still the 24th at UTC-7.
//! let instant = CanonicalTimestamp::from_ymd_hms_micros(2026, 11, 25, 1, 0, 0, 0).unwrap();
//! let utah = WorthQueryTimeZone::fixed(WorthQueryUtcOffset::from_hours_east(-7).unwrap());
//! assert_eq!(utah.date_at(instant), Ok(CanonicalDate::from_ymd(2026, 11, 24).unwrap()));
//!
//! // A desktop app loads the system zone once and asks it for today.
//! if let Ok(zone) = WorthQueryTimeZone::load_system() {
//!     let _today: CanonicalDate = zone.today().unwrap();
//! }
//! ```
pub use crate::local_calendar::{
    WorthQueryLocalCalendarDenial, WorthQueryTimeZone, WorthQueryUtcOffset,
};
