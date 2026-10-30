//! Genuine failed-ingest cleanup can coexist with selected NoRelease custody.

#[allow(unused_imports)]
use crate::production_entry::no_release_failed_ingest::*;

#[test]
fn selected_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(false, false);
}

#[test]
fn selected_tier_no_release_with_failed_ingest_drop_opens_serving() {
    exercise(true, false);
}

#[test]
fn changed_failed_ingest_control_after_seal_denies_serving_open() {
    exercise(false, true);
}
