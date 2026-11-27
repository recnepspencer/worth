//! W9A-2 standing grants: a QMS site role is a first-class standing grant
//! with no workflow, parent, grantor or delegation-limit field. Its validity is
//! a UTC instant or a local calendar day judged at the one trusted admission
//! instant, deactivating its site revokes it without touching the grant, and
//! no operation can link its own principal as a grantee. Every court runs on a
//! Directory Home and reopens it.
mod assign;
mod courts;
mod grants;
mod program;
mod schema;

use std::path::PathBuf;

use worth_query_decl::facade::application_schema::{CanonicalDate, CanonicalTimestamp};

/// The two sites; alice holds a review and an approval grant over each.
pub const SITES: [&str; 2] = ["north", "south"];

/// The validity bounds every seeded grant carries.
#[derive(Clone, Copy)]
pub struct Window {
    pub valid_from: CanonicalTimestamp,
    pub valid_until: CanonicalTimestamp,
    pub first_day: CanonicalDate,
    pub last_day: CanonicalDate,
}

pub fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> CanonicalTimestamp {
    CanonicalTimestamp::from_ymd_hms_micros(year, month, day, hour, minute, 0, 0).unwrap()
}

pub fn day(year: i32, month: u8, day: u8) -> CanonicalDate {
    CanonicalDate::from_ymd(year, month, day).unwrap()
}

pub fn micros(instant: CanonicalTimestamp, delta: i64) -> CanonicalTimestamp {
    CanonicalTimestamp {
        micros_since_unix_epoch: instant.micros_since_unix_epoch + delta,
    }
}

/// November 2026: UTC review from the 1st through the end of the 26th, and
/// local approval days the 1st through the 26th.
pub fn november() -> Window {
    Window {
        valid_from: at(2026, 11, 1, 0, 0),
        valid_until: micros(at(2026, 11, 27, 0, 0), -1),
        first_day: day(2026, 11, 1),
        last_day: day(2026, 11, 26),
    }
}

fn temp_home_path(label: &str) -> PathBuf {
    let mut path =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("standing-grant-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}
