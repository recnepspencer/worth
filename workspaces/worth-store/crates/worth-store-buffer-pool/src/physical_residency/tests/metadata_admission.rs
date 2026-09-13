use super::*;

const CANONICAL_FRAME_METADATA_BYTES: u64 = 3 * 1024 * 1024;
const MAX_EXPECTED_ALLOCATED_METADATA_BYTES: u64 = 11 * 256 * 1024;

#[test]
fn canonical_store_metadata_envelope_admits_full_frame_capacity() {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let limits = PhysicalResidencyLimits::builder()
        .total_bytes(nonzero_bytes(384 * 1024 * 1024))
        .resident_bytes(nonzero_bytes(64 * 1024 * 1024))
        .metadata_bytes(nonzero_bytes(CANONICAL_FRAME_METADATA_BYTES))
        .frame_entries(nonzero_count(4096))
        .pinned_frames(nonzero_count(256))
        .pin_leases(nonzero_count(512))
        .dirty_frames(nonzero_count(64))
        .dirty_replacement_bytes(nonzero_bytes(64 * 1024 * 1024))
        .operation_bytes(nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Recovery, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Scrub, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Verification, nonzero_bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Blob, nonzero_bytes(256 * 1024 * 1024))
        .speculative_frames(Speculation::Prefetch, nonzero_count(256))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(256))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(64))
        .admit(NonZeroU64::MIN)
        .unwrap();
    let pool = PhysicalResidencyPool::open(store(15), limits)
        .expect("the canonical Store metadata envelope must admit its declared frame capacity");
    let allocated = pool.counters().metadata_bytes();
    assert!(
        allocated <= MAX_EXPECTED_ALLOCATED_METADATA_BYTES,
        "allocated frame metadata {allocated} exceeds the guarded bound {MAX_EXPECTED_ALLOCATED_METADATA_BYTES}"
    );
}

#[test]
fn impossible_entry_metadata_is_denied_before_hash_table_allocation() {
    let identity = store(14);
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let limits = PhysicalResidencyLimits::builder()
        .total_bytes(nonzero_bytes(2049))
        .resident_bytes(nonzero_bytes(1024))
        .metadata_bytes(nonzero_bytes(1))
        .frame_entries(nonzero_count(u32::MAX))
        .pinned_frames(nonzero_count(1))
        .pin_leases(nonzero_count(1))
        .dirty_frames(nonzero_count(1))
        .dirty_replacement_bytes(nonzero_bytes(1024))
        .operation_bytes(nonzero_bytes(1))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(1))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(1))
        .scope_bytes(Scope::Recovery, nonzero_bytes(1))
        .scope_bytes(Scope::Scrub, nonzero_bytes(1))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(1))
        .scope_bytes(Scope::Verification, nonzero_bytes(1))
        .scope_bytes(Scope::Blob, nonzero_bytes(1))
        .speculative_frames(Speculation::Prefetch, nonzero_count(1))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(1))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(1))
        .admit(NonZeroU64::MIN)
        .unwrap();
    assert_eq!(
        PhysicalResidencyPool::open(identity, limits).unwrap_err(),
        PhysicalResidencyDenial::MetadataBudgetExceeded
    );
}
