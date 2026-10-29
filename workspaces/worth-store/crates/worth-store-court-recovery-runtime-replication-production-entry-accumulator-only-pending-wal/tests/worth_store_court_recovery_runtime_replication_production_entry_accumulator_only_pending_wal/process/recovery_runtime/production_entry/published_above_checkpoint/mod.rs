//! Objects published above the selected checkpoint by a process killed
//! before the next one. Their index maintenance filled whole inline pages
//! and retired every record on them, so recovery meets WAL images of pages
//! the selected root no longer routes.

#[allow(unused_imports)] use crate::production_entry::published_above_checkpoint::*;
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving, worth_store_testkit_recovery_runtime::production_entry::pending_successor_above_history, worth_store_testkit_recovery_runtime::production_entry::pending_wal_world, crate::production_entry::published_above_checkpoint};
use {worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::PendingWalWorld};
use {std::fs};

#[test]
fn one_24_chunk_object_published_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::TwentyFourChunks, Tail::Idle, 0xe1);
}

#[test]
fn three_2_chunk_objects_published_above_the_checkpoint_recover() {
    assert_recovers_reads_back_and_reopens(Workload::ThreeSmallObjects, Tail::Idle, 0xe2);
}

#[test]
fn one_66_chunk_object_published_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::SixtySixChunks, Tail::Idle, 0xe3);
}

#[test]
fn one_24_chunk_object_under_a_pending_release_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::TwentyFourChunks, Tail::PendingRelease, 0xe4);
}

#[test]
fn three_2_chunk_objects_under_a_pending_release_above_the_checkpoint_recover() {
    assert_recovers_reads_back_and_reopens(Workload::ThreeSmallObjects, Tail::PendingRelease, 0xe5);
}

#[test]
fn one_66_chunk_object_under_a_pending_release_above_the_checkpoint_recovers() {
    assert_recovers_reads_back_and_reopens(Workload::SixtySixChunks, Tail::PendingRelease, 0xe6);
}
#[cfg(all())]
#[path = "damaged_history/mod.rs"]
pub(crate) mod damaged_history;
#[cfg(all())]
#[path = "limit_sweep/mod.rs"]
pub(crate) mod limit_sweep;
#[cfg(all())]
#[path = "manifest_entry_limit/mod.rs"]
pub(crate) mod manifest_entry_limit;
