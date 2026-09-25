//! Compile probes for the Motion owner's edits to its tracks.
//!
//! See `crate::compile_probes` for how the cases are run.
#![expect(dead_code, reason = "compile probes are checked, never called")]

use super::super::track_sampling::UiPresentationTrackState;
use super::track_table::UiMotionTrackTable;
use crate::runtime::motion::UiMotionTargetIdentity;

// expect: compiles
#[cfg(worth_ui_compile_probe = "track-table-install")]
fn track_table_install(
    table: &mut UiMotionTrackTable,
    target: UiMotionTargetIdentity,
    state: UiPresentationTrackState,
) {
    table.install(target, state);
}

// expect: E0616
#[cfg(worth_ui_compile_probe = "track-table-direct")]
fn track_table_direct(
    table: &mut UiMotionTrackTable,
    target: UiMotionTargetIdentity,
    state: UiPresentationTrackState,
) {
    table.tracks.insert(target, state);
}
