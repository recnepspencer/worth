#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_reclaim_crash::*;

#[test]
fn selected_manifest_kill_retains_failed_ingest_payload_without_a_drop() {
    assert_killed_reclaim("crash-reclaim-manifest", false);
}

#[test]
fn selected_reservation_kill_without_descriptor_does_not_strand_payload() {
    assert_killed_reclaim("crash-reclaim-reservation", false);
}

#[test]
fn selected_reservation_still_cleans_after_lease_expiry_and_wal_pruning() {
    assert_killed_reclaim(AGED_RESERVATION, false);
}

#[test]
fn durable_descriptor_wal_kill_replays_exact_failed_ingest_drop() {
    assert_killed_reclaim("crash-reclaim-wal", true);
}
