//! Composition guards run as part of `cargo test` for this workspace.
//!
//! Body shared with every other workspace and with CI — see
//! `scripts/ci/composition_guard_test.rs`.

#[path = "../../../../../../scripts/ci/composition_guard_test.rs"]
mod composition_guard_test;
