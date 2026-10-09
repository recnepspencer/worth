use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::PhysicalWalOpenFailure;
use worth_store_wal::{
    inspect_complete_wal_segment, WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId,
    WalTopologyDenialKind,
};

use super::{
    build_three_segment_inventory, read_u64, reopen_failure, segment_path, shift_segment_lsns,
    wal_policy, SEGMENT_BYTES,
};

#[test]
fn contiguous_outer_topology_rejects_a_resealed_invalid_inner_member_without_repair() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory(&store_root);
    let first = segment_path(&store_root, 1, 1);
    reseal_with_invalid_inner_redo(&first);
    let before = std::fs::read(&first).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 3)),
        PhysicalWalOpenFailure::MemberPayloadRejected,
    );
    assert_eq!(std::fs::read(&first).unwrap(), before);
}

#[test]
fn earlier_invalid_inner_member_does_not_hide_a_later_outer_lsn_gap_or_repair_media() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    build_three_segment_inventory(&store_root);
    let first = segment_path(&store_root, 1, 1);
    let third = segment_path(&store_root, 3, 1);
    reseal_with_invalid_inner_redo(&first);
    shift_segment_lsns(&third, 1);
    let first_before = std::fs::read(&first).unwrap();
    let third_before = std::fs::read(&third).unwrap();

    assert_eq!(
        reopen_failure(&store_root, wal_policy(SEGMENT_BYTES, 3)),
        PhysicalWalOpenFailure::Topology(WalTopologyDenialKind::Gap),
    );
    assert_eq!(std::fs::read(&first).unwrap(), first_before);
    assert_eq!(std::fs::read(&third).unwrap(), third_before);
}

fn reseal_with_invalid_inner_redo(path: &Path) {
    let mut bytes = std::fs::read(path).unwrap();
    let payload_start = 116;
    let payload_end = payload_start + read_u64(&bytes, 44) as usize;
    let binding_len = read_u64(&bytes, payload_start) as usize;
    let redo_field = payload_start + 8 + binding_len;
    let redo_len = read_u64(&bytes, redo_field) as usize;
    let redo_start = redo_field + 8;
    let domain_len = read_u64(&bytes, redo_start) as usize;
    let domain_start = redo_start + 8;
    assert_eq!(
        &bytes[domain_start..domain_start + domain_len],
        b"store.physical.wal.canonical-redo.v3"
    );
    assert_eq!(redo_start + redo_len, payload_end);
    bytes[domain_start] ^= 1;
    let payload_digest = Sha256::digest(&bytes[payload_start..payload_end]);
    bytes[84..116].copy_from_slice(&payload_digest);
    let digest = Sha256::digest(&bytes[..payload_end]);
    bytes[payload_end..payload_end + 32].copy_from_slice(&digest);
    let identity = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(1).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
    );
    inspect_complete_wal_segment(identity, &bytes)
        .expect("resealed outer WAL frame must remain independently valid");
    std::fs::write(path, bytes).unwrap();
}
