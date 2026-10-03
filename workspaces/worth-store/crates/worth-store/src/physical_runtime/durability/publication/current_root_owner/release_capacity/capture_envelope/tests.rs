//! The capture envelope against the real shared Recovery pool.
use super::super::{
    backing::ReleasePublicationAllocationOwner, ReleaseCertificateCapacityDenial as Denial,
    SelectedReleaseCustodyLedger,
};
use super::*;
use crate::physical_runtime::{
    lifecycle::LifecycleCoordinator, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryRejoinResidentDenial, RuntimeIdentity,
};
use std::num::{NonZeroU32, NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyLimits,
    PhysicalSpeculativeWorkKind as Speculation,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

mod boundary;

const POOL: u64 = 65_536;

struct Fixture {
    owner: ReleasePublicationAllocationOwner,
    ports: crate::physical_runtime::record_serving::RecordFramePorts,
    ceiling: PhysicalRecoveryAllocationAdmission,
}

fn fixture() -> Fixture {
    fixture_with(POOL, 0)
}

/// A genuine Store pool whose ordinary scopes keep `headroom` bytes back.
fn fixture_with(pool: u64, headroom: u64) -> Fixture {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([118; 16]).unwrap(),
    )
    .published_identity();
    let bytes = NonZeroU64::new(pool).unwrap();
    let frames = NonZeroU32::new(4).unwrap();
    let limits = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(4 * pool).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(bytes)
        .dirty_replacement_bytes(bytes)
        .frame_entries(frames)
        .pinned_frames(frames)
        .pin_leases(frames)
        .dirty_frames(frames)
        .operation_bytes(bytes)
        .scope_bytes(Scope::ForegroundRead, bytes)
        .scope_bytes(Scope::ForegroundWrite, bytes)
        .scope_bytes(Scope::Recovery, bytes)
        .scope_bytes(Scope::Scrub, bytes)
        .scope_bytes(Scope::Maintenance, bytes)
        .scope_bytes(Scope::Verification, bytes)
        .scope_bytes(Scope::Blob, bytes)
        .speculative_frames(Speculation::Prefetch, frames)
        .speculative_frames(Speculation::ReadAhead, frames)
        .speculative_frames(Speculation::WriteBehind, frames)
        .progress_headroom_bytes(headroom)
        .admit(NonZeroU64::MIN)
        .unwrap();
    let ports =
        crate::physical_runtime::record_serving::RecordFramePorts::bounded(store, limits).unwrap();
    let ceiling = PhysicalRecoveryAllocationAdmission::new(store, pool);
    let lifecycle = LifecycleCoordinator::admitted();
    lifecycle.progress_to_media_owned();
    let generation = lifecycle.progress_to_record_serving().generation;
    let owner = ReleasePublicationAllocationOwner::new(
        ports.clone(),
        ceiling,
        RuntimeIdentity::from_reopened(NonZeroU64::MIN),
        generation,
        lifecycle.observation_state(),
        0,
    );
    Fixture {
        owner,
        ports,
        ceiling,
    }
}

fn admitted(
    fixture: &Fixture,
    envelope: CheckpointCaptureEnvelope,
) -> Arc<SealedCheckpointStorage> {
    let mut custody = None;
    fixture
        .owner
        .fund(&mut custody, fixture.ceiling, envelope.bytes())
        .unwrap();
    Arc::new(SealedCheckpointStorage::from_prepared(
        envelope.prepare(custody.unwrap()),
    ))
}

fn active(fixture: &Fixture) -> u64 {
    fixture.ports.counters().active_operation_bytes()
}

#[test]
fn envelope_covers_every_capture_buffer_and_dies_with_them() {
    let fixture = fixture();
    let ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    for tier in [false, true] {
        let envelope = ledger.checkpoint_capture_envelope(None, tier, 0).unwrap();
        assert_eq!(envelope.records, 1 + usize::from(tier));
        let mut custody = None;
        fixture
            .owner
            .fund(&mut custody, fixture.ceiling, envelope.bytes())
            .unwrap();
        assert_eq!(active(&fixture), envelope.bytes());
        let preparation = envelope.prepare(custody.unwrap());
        let observer = &preparation.observer;
        let fold = &preparation.fold;
        let buffers = observer.bytes.capacity()
            + observer.records.capacity() * std::mem::size_of::<CertificateRange>()
            + observer.scratch.capacity()
            + observer.frame_scratch.capacity()
            + fold.heads.owned_heap_bytes().unwrap() as usize
            + fold.batches.capacity() * std::mem::size_of::<ReleaseCheckpointBatchV1>()
            + fold.scratch.capacity()
            + IN_FLIGHT_COMMANDS * COMMAND_BYTES
            + SEALED_STORAGE_BYTES;
        assert!(buffers as u64 + SHARED_CUSTODY_BYTES <= envelope.bytes());
        drop(Arc::new(SealedCheckpointStorage::from_prepared(
            preparation,
        )));
        assert_eq!(active(&fixture), 0);
    }
}

#[test]
fn undersized_budget_denies_the_envelope_before_any_capture_allocation() {
    let fixture = fixture();
    let envelope = SelectedReleaseCustodyLedger::trusted_genesis()
        .checkpoint_capture_envelope(None, false, 0)
        .unwrap();
    let held = fixture
        .ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(POOL - envelope.bytes() + 1).unwrap(),
        )
        .unwrap();
    let mut custody = None;
    let Err(Denial::Resident(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause))) =
        fixture
            .owner
            .fund(&mut custody, fixture.ceiling, envelope.bytes())
    else {
        panic!("held Recovery pressure must deny the whole envelope");
    };
    assert_eq!(cause.pressure().unwrap().requested(), envelope.bytes());
    assert!(custody.is_none());
    assert_eq!(active(&fixture), POOL - envelope.bytes() + 1);
    drop(held);
    assert!(matches!(
        fixture.owner.fund(&mut custody, fixture.ceiling, POOL + 1),
        Err(Denial::Resident(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { .. }
        ))
    ));
}

#[test]
fn a_capture_consumes_the_standing_reservation_without_new_bytes() {
    let fixture = fixture();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger
        .reserve_capture_custody(&fixture.owner, fixture.ceiling, None)
        .unwrap();
    let reserved = active(&fixture);
    assert_eq!(ledger.capture_custody_bytes(), Some(reserved));
    assert_eq!(
        ledger.capture_custody_requirement(None, 0).unwrap(),
        reserved
    );
    let capture = |ledger: &mut SelectedReleaseCustodyLedger, tier| {
        Arc::new(SealedCheckpointStorage::from_prepared(
            ledger
                .prepare_capture(&fixture.owner, fixture.ceiling, tier)
                .unwrap(),
        ))
    };
    // The reservation already holds a tier slot: no capture grows it.
    let first = capture(&mut ledger, true);
    assert_eq!(active(&fixture), reserved);
    // Only a capture that overlaps a still-live one funds its own envelope.
    let envelope = ledger.checkpoint_capture_envelope(None, false, 0).unwrap();
    let second = capture(&mut ledger, false);
    assert_eq!(active(&fixture), reserved + envelope.bytes());
    drop(second);
    assert_eq!(active(&fixture), reserved);
    // A cancelled capture leaves the reservation whole for the next one.
    drop(first);
    assert_eq!(active(&fixture), reserved);
    drop(capture(&mut ledger, false));
    assert_eq!(active(&fixture), reserved);
    drop(ledger);
    assert_eq!(active(&fixture), 0);
}

#[test]
fn an_uncommitted_capture_returns_its_whole_envelope() {
    let fixture = fixture();
    let envelope = SelectedReleaseCustodyLedger::trusted_genesis()
        .checkpoint_capture_envelope(None, true, 0)
        .unwrap();
    let first = admitted(&fixture, envelope);
    let lease = first.lease_fold().unwrap();
    // A held snapshot never blocks the next capture: each owns its envelope.
    let second = admitted(&fixture, envelope);
    assert_eq!(active(&fixture), 2 * envelope.bytes());
    drop(lease);
    drop(first);
    assert_eq!(active(&fixture), envelope.bytes());
    drop(second);
    assert_eq!(active(&fixture), 0);
}

#[test]
fn every_capture_envelope_holds_the_pin_scan_bound() {
    let ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    let scan = 4_096;
    for tier in [false, true] {
        let without = ledger.checkpoint_capture_envelope(None, tier, 0).unwrap();
        let with = ledger
            .checkpoint_capture_envelope(None, tier, scan)
            .unwrap();
        assert_eq!(with.bytes(), without.bytes() + scan);
    }
    assert_eq!(
        ledger.capture_custody_requirement(None, scan).unwrap(),
        ledger.capture_custody_requirement(None, 0).unwrap() + scan
    );
}
