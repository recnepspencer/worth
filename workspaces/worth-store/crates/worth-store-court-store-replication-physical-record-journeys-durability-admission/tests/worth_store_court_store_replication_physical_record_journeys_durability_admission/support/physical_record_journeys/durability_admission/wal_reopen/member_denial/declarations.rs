pub(crate) use std::path::Path;

pub(crate) use worth_store::physical_runtime::PhysicalWalOpenFailure;
pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store_wal::inspect_complete_wal_segment, worth_store_wal::WalSegmentArtifactIdentity,
    worth_store_wal::WalSegmentGeneration, worth_store_wal::WalSegmentId,
    worth_store_wal::WalTopologyDenialKind,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::build_three_segment_inventory,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::read_u64,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::reopen_failure,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::segment_path,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::shift_segment_lsns,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::wal_policy,
    worth_store_testkit_store::physical_record_journeys::durability_admission::wal_reopen::SEGMENT_BYTES,
};

pub(crate) fn reseal_with_invalid_inner_redo(path: &Path) {
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
    let payload_digest = sha2::Sha256::digest(&bytes[payload_start..payload_end]);
    bytes[84..116].copy_from_slice(&payload_digest);
    let digest = sha2::Sha256::digest(&bytes[..payload_end]);
    bytes[payload_end..payload_end + 32].copy_from_slice(&digest);
    let identity = worth_store_wal::WalSegmentArtifactIdentity::new(
        worth_store_wal::WalSegmentId::new(1).unwrap(),
        worth_store_wal::WalSegmentGeneration::new(1).unwrap(),
    );
    inspect_complete_wal_segment(identity, &bytes)
        .expect("resealed outer WAL frame must remain independently valid");
    std::fs::write(path, bytes).unwrap();
}
