#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_crash::*;

#[test]
fn declaration_root_publication_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-declaration", std::time::Duration::from_secs(90));
    assert_reopened(&world, false, 0);
}

#[test]
fn selected_chunk_claim_survives_kill_without_inventing_generation() {
    let world = kill_at("crash-chunk", std::time::Duration::from_secs(90));
    assert_reopened(&world, false, 1);
}

#[test]
fn wal_durable_generation_rolls_forward_after_kill_before_root_publication() {
    let world = kill_at("crash-generation-wal", std::time::Duration::from_secs(300));
    assert_reopened(&world, true, CRASH_OBJECT_CHUNKS);
}
