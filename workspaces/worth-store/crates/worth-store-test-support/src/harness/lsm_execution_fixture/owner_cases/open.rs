use worth_store_lsm_authority::{
    open_lsm_membership, reopen_lsm_membership_from_store, LsmMembershipOwnerCaseObservation,
};
use worth_store_wal::{BlobWalRecordKind, WalArtifactInventory, WalFrameArtifactObservation};

use super::super::begin_durability_fixture;
use super::published_replacement::published_replacement;
use super::world;

pub(super) fn observe() -> Vec<LsmMembershipOwnerCaseObservation> {
    vec![
        admitted(),
        canonical_key_required(),
        durable_record_binding_mismatch(),
        unsupported_record_kind(),
        membership_ambiguous(),
        membership_stale(),
        manifest_membership_mismatch(),
        replacement_output_mismatch(),
        persisted_membership_artifact_invalid(),
        io(),
    ]
}

fn admitted() -> LsmMembershipOwnerCaseObservation {
    let world = world::complete_membership();
    open(&world.anchor)
}

fn canonical_key_required() -> LsmMembershipOwnerCaseObservation {
    let world = world::complete_membership();
    let foreign_scope =
        worth_store_security::admitted_tenant_wal_checkpoint_security_scope_for_layout_partition_test();
    open_lsm_membership(&world.anchor, foreign_scope.witnesses()).owner_case_observation()
}

fn durable_record_binding_mismatch() -> LsmMembershipOwnerCaseObservation {
    let world = world::complete_membership();
    use std::io::{Read, Seek, SeekFrom};
    let mut bytes = vec![0; world.anchor.payload_bytes() as usize];
    let mut artifact = std::fs::File::open(world.anchor.path()).unwrap();
    artifact
        .seek(SeekFrom::Start(world.anchor.payload_offset()))
        .unwrap();
    artifact.read_exact(&mut bytes).unwrap();
    bytes[b"worth-store:wal-lsm-membership:v1 ".len()] ^= 1;
    world::persist_untrusted_artifact(44, &bytes);
    open(&world.anchor)
}

fn unsupported_record_kind() -> LsmMembershipOwnerCaseObservation {
    begin_durability_fixture();
    let (_, key) = world::admission_and_key(81);
    let (_, anchor) = world::admitted_record(key, 81, BlobWalRecordKind::RootCandidate);
    open(&anchor)
}

fn membership_ambiguous() -> LsmMembershipOwnerCaseObservation {
    begin_durability_fixture();
    let (_, key) = world::admission_and_key(91);
    let (_, anchor) = world::admitted_record(key, 90, BlobWalRecordKind::LsmValue);
    world::admitted_record(key, 91, BlobWalRecordKind::LsmValue);
    open(&anchor)
}

/// The activation survives while every input record it retired is gone.
fn membership_stale() -> LsmMembershipOwnerCaseObservation {
    let world = published_replacement();
    let store = inventory_for(&world.anchor);
    std::fs::remove_file(&world.segment_path).unwrap();
    reopen(store)
}

/// The activation names a complete selection, but the tombstone input is gone.
fn manifest_membership_mismatch() -> LsmMembershipOwnerCaseObservation {
    let world = published_replacement();
    world.truncate_segment_at(world.record_frame_offsets[2]);
    open(&world.anchor)
}

/// Every input survives, but the output frame the activation names is gone.
fn replacement_output_mismatch() -> LsmMembershipOwnerCaseObservation {
    let world = published_replacement();
    world.truncate_segment_at(world.output_frame_offset);
    open(&world.anchor)
}

fn persisted_membership_artifact_invalid() -> LsmMembershipOwnerCaseObservation {
    let world = world::complete_membership();
    let body = b"worth-store:wal-lsm-membership:v1 0";
    let malformed = format!(
        "{} {:016x}",
        std::str::from_utf8(body).unwrap(),
        checksum(body)
    );
    world::persist_untrusted_artifact(44, malformed.as_bytes());
    open(&world.anchor)
}

fn io() -> LsmMembershipOwnerCaseObservation {
    let world = world::complete_membership();
    let store = inventory_for(&world.anchor);
    let root = world
        .anchor
        .path()
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let current_scope =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    reopen_lsm_membership_from_store(store, current_scope.witnesses()).owner_case_observation()
}

fn open(anchor: &WalFrameArtifactObservation) -> LsmMembershipOwnerCaseObservation {
    let current_scope =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    open_lsm_membership(anchor, current_scope.witnesses()).owner_case_observation()
}

fn reopen(store: WalArtifactInventory) -> LsmMembershipOwnerCaseObservation {
    let current_scope =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    reopen_lsm_membership_from_store(store, current_scope.witnesses()).owner_case_observation()
}

fn inventory_for(anchor: &WalFrameArtifactObservation) -> WalArtifactInventory {
    let root = anchor
        .path()
        .parent()
        .and_then(std::path::Path::parent)
        .expect("fixture WAL root");
    WalArtifactInventory::open(
        root,
        anchor.scope().segment_id(),
        anchor.scope().generation(),
    )
    .expect("fixture WAL inventory")
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
