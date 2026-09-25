//! Compile probes for Scroll group translations by truth status, and for
//! committing only what a witness displayed.
//!
//! See `crate::compile_probes` for how the cases are run.
#![expect(dead_code, reason = "compile probes are checked, never called")]

use super::group_offset::UiPublishedToAcceptedTranslation;

// expect: compiles
#[cfg(worth_ui_compile_probe = "group-move-published")]
fn group_move_published(
    first: UiPublishedToAcceptedTranslation,
    then: UiPublishedToAcceptedTranslation,
) {
    let _sum = first.then(then);
}

// expect: E0308
#[cfg(worth_ui_compile_probe = "group-move-displayed")]
fn group_move_displayed(
    first: UiPublishedToAcceptedTranslation,
    then: super::group_offset::UiDisplayedToAcceptedTranslation,
) {
    let _sum = first.then(then);
}

// expect: compiles
#[cfg(worth_ui_compile_probe = "group-update-commit-displayed")]
fn group_update_commit_displayed(displayed: super::acceptance::UiDisplayedScrollGroupMotion) {
    displayed.commit();
}

// expect: E0599
#[cfg(worth_ui_compile_probe = "group-update-commit-prepared")]
fn group_update_commit_prepared(prepared: super::UiScrollGroupMotionUpdate) {
    prepared.commit();
}
