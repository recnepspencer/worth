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
//!
//! // With `TZ` unset the platform default applies: `/etc/localtime`, or on
//! // Windows the registry's zone record, which parses on any platform.
//! assert_eq!(
//!     WorthQueryTimeZone::from_tz_environment(None),
//!     WorthQueryTimeZone::from_tz_variable(":"),
//! );
//! let mountain = WorthQueryTimeZone::from_windows_registry_query(
//!     "    Bias    REG_DWORD    0x1a4\r\n\
//!      \x20   StandardBias    REG_DWORD    0x0\r\n\
//!      \x20   DaylightBias    REG_DWORD    0xffffffc4\r\n\
//!      \x20   StandardStart    REG_BINARY    00000B00000001000200000000000000\r\n\
//!      \x20   DaylightStart    REG_BINARY    00000300000002000200000000000000\r\n",
//! )
//! .unwrap();
//! let july = CanonicalTimestamp::from_ymd_hms_micros(2026, 7, 1, 12, 0, 0, 0).unwrap();
//! assert_eq!(mountain.offset_at(july).seconds_east(), -6 * 3_600);
//! assert_eq!(mountain.offset_at(instant).seconds_east(), -7 * 3_600);
//! ```
//!
//! Commit-time rules convert the operation's trusted admission instant, never
//! a caller's date. `DecisionReader::admitted_at()` and
//! `WorthQueryApplicationInvariantContext::admitted_at()` return the same
//! reading of the Home's runtime time source; a held source makes it
//! deterministic in tests and moves only when told to:
//!
//! ```
//! use std::time::Duration;
//! use worth_query_host::facade::declaration::application_schema::{
//!     CanonicalDate, CanonicalTimestamp,
//! };
//! use worth_query_host::facade::local_calendar::WorthQueryTimeZone;
//! use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;
//!
//! // Pass a clone to `WorthQueryHomeConfig::with_authorization_time_source`.
//! let clock = WorthQueryFixedRuntimeTimeSource::new(
//!     CanonicalTimestamp::from_ymd_hms_micros(2026, 11, 27, 1, 30, 0, 0).unwrap(),
//! );
//! // `from_tz_variable(":America/Denver")` loads the same rules from the zone database.
//! let mountain = WorthQueryTimeZone::from_tz_variable("MST7MDT,M3.2.0,M11.1.0").unwrap();
//! // 01:30Z on the 27th is still 18:30 on the 26th in Mountain time.
//! assert_eq!(mountain.date_at(clock.now()), Ok(CanonicalDate::from_ymd(2026, 11, 26).unwrap()));
//! clock.advance(Duration::from_secs(5 * 3_600 + 30 * 60)).unwrap();
//! assert_eq!(mountain.date_at(clock.now()), Ok(CanonicalDate::from_ymd(2026, 11, 27).unwrap()));
//! ```
//!
//! A declarative creation body stamps that same instant into a required
//! `CanonicalTimestamp` field of the row it creates, in the same commit and
//! with no caller input, including a workflow-enrolled birth:
//!
//! ```
//! use worth_query_host::facade::declaration::application_schema::{
//!     fragment::RecordSelection, ApplicationOperationBodyAuthoringDenial,
//!     ApplicationOperationBodyCreationBuilder, CanonicalTimestamp,
//!     CanonicalTimestampApplicationValueBinding,
//! };
//!
//! fn stamp_created_at<S, O, I, E, F>(
//!     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>,
//!     created_at: F,
//! ) -> Result<ApplicationOperationBodyCreationBuilder<S, O, I, E>, ApplicationOperationBodyAuthoringDenial>
//! where
//!     F: RecordSelection<Entity = E, Value = CanonicalTimestamp,
//!         Binding = CanonicalTimestampApplicationValueBinding>,
//! {
//!     // The handler's `admitted_at()` and every commit invariant's
//!     // `admitted_at()` observe exactly this stamp.
//!     creation.initialize_from_admission_time(created_at)
//! }
//! ```
pub use crate::local_calendar::{
    WorthQueryLocalCalendarDenial, WorthQueryTimeZone, WorthQueryUtcOffset,
};
