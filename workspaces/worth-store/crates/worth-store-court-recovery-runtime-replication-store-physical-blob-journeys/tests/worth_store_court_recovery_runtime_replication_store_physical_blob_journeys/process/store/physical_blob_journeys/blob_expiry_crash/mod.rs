#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_expiry_crash::*;

#[test]
fn expiry_wal_durable_before_root_replacement_replays_one_terminal() {
    assert_killed_expiry("crash-expiry-wal");
}

#[test]
fn expiry_root_replaced_before_completion_reopens_one_terminal() {
    assert_killed_expiry("crash-expiry-root");
}
