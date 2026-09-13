use super::*;
use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};
use worth_store_physical_format::{
    store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    },
    RecordArtifactFile, RecordFrameCoordinate,
};

const TEST_FRAME_METADATA_BYTES: u64 = 8192;

#[path = "tests/allocation_events/mod.rs"]
mod allocation_events;
#[path = "tests/candidate_artifact_alias.rs"]
mod candidate_artifact_alias;
#[path = "tests/candidate_concurrency.rs"]
mod candidate_concurrency;
#[path = "tests/candidate_contract.rs"]
mod candidate_contract;
#[path = "tests/candidate_identity_conflicts.rs"]
mod candidate_identity_conflicts;
#[path = "tests/candidate_window.rs"]
mod candidate_window;
#[path = "tests/clean_to_dirty.rs"]
mod clean_to_dirty;
#[path = "tests/dirty_generation_capture.rs"]
mod dirty_generation_capture;
#[path = "tests/eviction_siege/mod.rs"]
mod eviction_siege;
#[path = "tests/frame_access/mod.rs"]
mod frame_access;
#[path = "tests/identity_transition.rs"]
mod identity_transition;
#[path = "tests/integrity_validation.rs"]
mod integrity_validation;
#[path = "tests/metadata_admission.rs"]
mod metadata_admission;
#[path = "tests/operation_allocation.rs"]
mod operation_allocation;
#[path = "tests/pin_lease_pressure.rs"]
mod pin_lease_pressure;
#[path = "tests/pressure_limits/mod.rs"]
mod pressure_limits;
#[path = "tests/shutdown.rs"]
mod shutdown;
#[path = "tests/speculation.rs"]
mod speculation;
#[path = "tests/speculation_limits/mod.rs"]
mod speculation_limits;
#[path = "tests/writeback_claim_exclusion.rs"]
mod writeback_claim_exclusion;
#[path = "tests/writeback_range_posture.rs"]
mod writeback_range_posture;

fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

fn coordinate(block: u64, length: u32) -> RecordFrameCoordinate {
    RecordFrameCoordinate::new(
        RecordArtifactFile::RootRoutingBlock {
            generation: 1,
            block,
        },
        0,
        length,
    )
    .unwrap()
}

fn limits(
    resident_bytes: u64,
    pinned_frames: u32,
    dirty_frames: u32,
    operation_bytes: u64,
    frame_entries: u32,
) -> PhysicalResidencyLimits {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let total_bytes = resident_bytes
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(operation_bytes))
        .and_then(|bytes| bytes.checked_add(TEST_FRAME_METADATA_BYTES))
        .unwrap();
    PhysicalResidencyLimits::builder()
        .total_bytes(nonzero_bytes(total_bytes))
        .resident_bytes(nonzero_bytes(resident_bytes))
        .metadata_bytes(nonzero_bytes(TEST_FRAME_METADATA_BYTES))
        .frame_entries(nonzero_count(frame_entries))
        .pinned_frames(nonzero_count(pinned_frames))
        .pin_leases(nonzero_count(pinned_frames))
        .dirty_frames(nonzero_count(dirty_frames))
        .dirty_replacement_bytes(nonzero_bytes(resident_bytes))
        .operation_bytes(nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Recovery, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Scrub, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Verification, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Blob, nonzero_bytes(operation_bytes))
        .speculative_frames(Speculation::Prefetch, nonzero_count(pinned_frames))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(pinned_frames))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(dirty_frames))
        .admit(NonZeroU64::MIN)
        .unwrap()
}

fn nonzero_bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}

fn allocation(
    pool: &PhysicalResidencyPool,
    scope: PhysicalOperationAllocationScope,
) -> OperationAllocationGrant {
    pool.begin_operation(scope, NonZeroU64::MIN).unwrap()
}

fn candidate_batch_bytes(candidate_count: usize) -> u64 {
    PhysicalResidencyPool::candidate_batch_operation_bytes(
        NonZeroUsize::new(candidate_count).expect("candidate test batch is nonempty"),
    )
    .expect("candidate test batch demand fits u64")
    .get()
}

fn candidate_allocation(
    pool: &PhysicalResidencyPool,
    candidate_count: usize,
) -> ForegroundWriteAllocationGrant {
    pool.begin_foreground_write_operation(
        NonZeroU64::new(candidate_batch_bytes(candidate_count)).unwrap(),
    )
    .unwrap()
}

fn candidate_batches_bytes(batch_candidate_counts: &[usize]) -> u64 {
    batch_candidate_counts
        .iter()
        .map(|count| candidate_batch_bytes(*count))
        .sum()
}

fn candidate_batches_allocation(
    pool: &PhysicalResidencyPool,
    batch_candidate_counts: &[usize],
) -> ForegroundWriteAllocationGrant {
    pool.begin_foreground_write_operation(
        NonZeroU64::new(candidate_batches_bytes(batch_candidate_counts)).unwrap(),
    )
    .unwrap()
}

fn writeback_claim(
    pool: &PhysicalResidencyPool,
    frames: &[PhysicalFrameKey],
) -> PhysicalWritebackClaim {
    let bytes = frames
        .iter()
        .map(|frame| u64::from(frame.coordinate().length()))
        .sum();
    let allocation = pool
        .begin_foreground_write_operation(nonzero_bytes(bytes))
        .unwrap();
    pool.claim_writeback(allocation, frames).unwrap()
}

fn expect_fault(
    pool: &PhysicalResidencyPool,
    allocation: &OperationAllocationGrant,
    key: PhysicalFrameKey,
) -> PhysicalFrameFaultOwner {
    match pool.access_frame(allocation, key).unwrap() {
        PhysicalFrameAccess::Fault(fault) => fault,
        PhysicalFrameAccess::Hit(_) => panic!("expected a cold frame fault, found a resident hit"),
        PhysicalFrameAccess::Coalesced(_) => {
            panic!("expected sole fault ownership, found a coalesced waiter")
        }
    }
}

fn expect_hit(
    pool: &PhysicalResidencyPool,
    allocation: &OperationAllocationGrant,
    key: PhysicalFrameKey,
) -> PhysicalFrameLease {
    match pool.access_frame(allocation, key).unwrap() {
        PhysicalFrameAccess::Hit(lease) => lease,
        PhysicalFrameAccess::Fault(_) => panic!("expected a resident hit, found a cold fault"),
        PhysicalFrameAccess::Coalesced(_) => {
            panic!("expected a resident hit, found a coalesced waiter")
        }
    }
}

fn fill(bytes: &mut [u8], value: u8) -> Result<(), ()> {
    bytes.fill(value);
    Ok(())
}

const READ_SCOPE: PhysicalOperationAllocationScope =
    PhysicalOperationAllocationScope::ForegroundRead;
const WRITE_SCOPE: PhysicalOperationAllocationScope =
    PhysicalOperationAllocationScope::ForegroundWrite;

#[test]
fn pinned_frames_are_never_selected_for_eviction() {
    let identity = store(10);
    let pool = PhysicalResidencyPool::open(identity, limits(64, 2, 1, 64, 3)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let first_key = PhysicalFrameKey::new(identity, coordinate(1, 32));
    let second_key = PhysicalFrameKey::new(identity, coordinate(2, 32));
    let first = expect_fault(&pool, &allocation, first_key)
        .load(|bytes| fill(bytes, 1))
        .unwrap();
    let second = expect_fault(&pool, &allocation, second_key)
        .load(|bytes| fill(bytes, 2))
        .unwrap();
    let third_key = PhysicalFrameKey::new(identity, coordinate(3, 32));
    assert_eq!(
        pool.access_frame(&allocation, third_key).unwrap_err(),
        PhysicalResidencyDenial::Pressure(PhysicalResidencyPressureDenial::new(
            identity,
            pool.incarnation(),
            PhysicalResidencyPressureDemand {
                dimension: PhysicalResidencyDimension::PinLeases,
                scope: READ_SCOPE,
                requested: 1,
                current: 2,
                limit: 2,
            },
        ),)
    );
    assert_eq!(&*first, &[1; 32]);
    assert_eq!(&*second, &[2; 32]);
    drop(first);
    let third = expect_fault(&pool, &allocation, third_key)
        .load(|bytes| fill(bytes, 3))
        .unwrap();
    assert_eq!(&*third, &[3; 32]);
    assert_eq!(pool.counters().evictions(), 1);
    assert_eq!(pool.counters().eviction_candidate_inspections(), 1);
}

#[test]
fn clean_identity_promotion_moves_the_cached_frame_atomically() {
    let identity = store(11);
    let pool = PhysicalResidencyPool::open(identity, limits(128, 2, 1, 64, 3)).unwrap();
    let read_allocation = allocation(&pool, READ_SCOPE);
    let write_allocation = allocation(&pool, WRITE_SCOPE);
    let old = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 4).unwrap();
    let candidate = RecordFrameCoordinate::new(
        RecordArtifactFile::CatalogCandidate { publication: 2 },
        0,
        4,
    )
    .unwrap();
    let old_key = PhysicalFrameKey::new(identity, old);
    let candidate_key = PhysicalFrameKey::new(identity, candidate);
    drop(
        expect_fault(&pool, &read_allocation, old_key)
            .load(|bytes| fill(bytes, 1))
            .unwrap(),
    );
    drop(
        expect_fault(&pool, &write_allocation, candidate_key)
            .load(|bytes| fill(bytes, 9))
            .unwrap(),
    );
    pool.promote_clean_identity(candidate_key, old_key).unwrap();
    let current = expect_hit(&pool, &read_allocation, old_key);
    assert_eq!(&*current, &[9; 4]);
    drop(current);
    let vanished = expect_fault(&pool, &read_allocation, candidate_key)
        .load(|bytes| fill(bytes, 7))
        .unwrap();
    assert_eq!(&*vanished, &[7; 4]);
    assert_eq!(pool.counters().identity_transitions(), 1);
    assert_eq!(
        pool.counters().source_loads(),
        3,
        "old identity, promoted clean source, and vanished-source refault each load once"
    );
}
