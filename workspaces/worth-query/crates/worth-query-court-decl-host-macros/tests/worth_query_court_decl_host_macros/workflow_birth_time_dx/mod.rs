//! W9B workflow birth: a body-only creation that also calls
//! `enroll_in_workflow` stamps its subject's required `created_at` with
//! `initialize_from_admission_time`. One commit creates the request, records
//! the trusted admission instant and starts the request's native workflow
//! instance; a commit-boundary invariant refuses any stamp other than its own
//! `context.admitted_at()`. The world is minimal but real: grants admit four
//! company capabilities, the vocabulary installs the actual creator with
//! `birth_operation`, and births run through the public request API. The
//! born request links to its company, so a company grant reaches it through
//! the governed subject read that `subject_lookup` reacquires it by.
mod birth;
mod control;
mod courts;
mod creator;
mod disclosure;
mod grants;
mod invariant;
mod program;
mod schema;
mod signing;
mod subject_control;
mod subject_lookup;
mod subject_pages;
mod workflow;

use worth_query_decl::facade::application_schema::CanonicalTimestamp;

/// The one company requests are born in.
pub const COMPANY: &str = "acme";

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}
