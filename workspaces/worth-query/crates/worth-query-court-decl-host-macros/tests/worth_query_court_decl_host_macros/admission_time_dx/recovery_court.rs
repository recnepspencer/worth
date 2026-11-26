//! Reopening a Directory Home recovers commits that were judged at their own
//! admission instant: recovery never re-judges them against a later clock.
use std::path::PathBuf;

use super::program::{open_directory, owner};
use super::schema::StampLot;
use super::{at, date, lot};
use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;

fn temp_home_path(label: &str) -> PathBuf {
    let mut path =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("admission-time-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn stamp_last_day(clock: &WorthQueryFixedRuntimeTimeSource, path: &std::path::Path, clean: bool) {
    let home = open_directory(path, clock.clone());
    let last_day = date(2026, 11, 26);
    home.session(owner(&home))
        .run(StampLot {
            lot: 1,
            effective_on: last_day,
            expires_on: last_day,
        })
        .expect("valid on its last Mountain day");
    if clean {
        home.close_clean().expect("Home closes");
    }
}

#[test]
fn a_directory_home_reopens_its_stamped_commit_after_the_lot_has_expired() {
    for clean in [true, false] {
        let path = temp_home_path(if clean { "clean" } else { "dropped" });
        let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 27, 1, 30));
        stamp_last_day(&clock, &path, clean);
        // Weeks later the lot is long expired, yet its commit was judged then.
        clock.set(at(2026, 12, 15, 12, 0));
        let home = open_directory(&path, clock.clone());
        let recovered = lot(&home);
        assert_eq!(recovered.expires_on, date(2026, 11, 26));
        assert_eq!(recovered.stamped_at, at(2026, 11, 27, 1, 30));
        home.close_clean().expect("Home closes");
    }
}
