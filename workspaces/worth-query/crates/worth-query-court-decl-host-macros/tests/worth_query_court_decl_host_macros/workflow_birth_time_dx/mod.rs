//! W9B workflow birth: a body-only creation that also calls
//! `enroll_in_workflow` stamps its subject's required `created_at` with
//! `initialize_from_admission_time`. One commit creates the request, records
//! the trusted admission instant and starts the request's native workflow
//! instance; a commit-boundary invariant refuses any stamp other than its own
//! `context.admitted_at()`. The world is minimal but real: grants admit four
//! company capabilities, the vocabulary installs the actual creator with
//! `birth_operation`, and births run through the public request API.
mod birth;
mod control;
mod courts;
mod creator;
mod grants;
mod invariant;
mod program;
mod schema;
mod signing;
mod workflow;

use worth_query_decl::facade::application_schema::CanonicalTimestamp;

/// The one company requests are born in.
pub const COMPANY: &str = "acme";

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}
