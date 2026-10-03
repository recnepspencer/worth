//! Focused ownership tests using a genuine bounded Store residency pool.

use super::*;
use crate::physical_runtime::lifecycle::LifecycleCoordinator;
use crate::physical_runtime::{PhysicalRecoveryRejoinResidentDenial, RuntimeIdentity};
use std::num::{NonZeroU32, NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyLimits,
    PhysicalSpeculativeWorkKind as Speculation,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

struct Fixture {
    owner: ReleasePublicationAllocationOwner,
    ports: crate::physical_runtime::record_serving::RecordFramePorts,
    ceiling: PhysicalRecoveryAllocationAdmission,
    lifecycle: LifecycleCoordinator,
}

fn fixture() -> Fixture {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([117; 16]).unwrap(),
    )
    .published_identity();
    let bytes = NonZeroU64::new(65_536).unwrap();
    let frames = NonZeroU32::new(4).unwrap();
    let limits = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(4 * bytes.get()).unwrap())
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
        .admit(NonZeroU64::MIN)
        .unwrap();
    let ports =
        crate::physical_runtime::record_serving::RecordFramePorts::bounded(store, limits).unwrap();
    let ceiling = PhysicalRecoveryAllocationAdmission::new(store, bytes.get());
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
        lifecycle,
    }
}

fn key() -> ReleaseCustodyHeadKeyV1 {
    ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap()
}

#[test]
fn pre_effect_control_funding_denial_allocates_no_ledger_backing() {
    let fixture = fixture();
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_535).unwrap())
        .unwrap();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    let Err(Denial::Resident(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause))) =
        ledger.prepare_control_backing(&fixture.owner, fixture.ceiling, 0)
    else {
        panic!("the real pool must deny pending control backing");
    };
    let pressure = cause.pressure().unwrap();
    assert_eq!(
        pressure.requested(),
        PENDING_CONTROL_BYTES + SHARED_CUSTODY_BYTES
    );
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 65_535);
    assert!(ledger.allocation_custody.is_none());
    assert_eq!(ledger.publication_backing_bytes(), Some(0));
    drop(collision);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn cancellation_and_drained_spare_capacity_keep_live_funding_until_disposal() {
    let fixture = fixture();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger
        .prepare_control_backing(&fixture.owner, fixture.ceiling, 64)
        .unwrap();
    let scratch = ledger
        .prepare_publication_backing(&fixture.owner, fixture.ceiling, 128, 64, key())
        .unwrap();
    assert!(
        scratch.bytes.capacity() >= DurablePhysicalRootManifest::maximum_encoding_scratch_bytes()
    );
    let retained = ledger.publication_backing_bytes().unwrap();
    assert!(retained > 0);
    let funded = fixture.ports.counters().active_operation_bytes();
    assert!(funded >= retained + 128 + 64 + PENDING_CONTROL_BYTES + SHARED_CUSTODY_BYTES);
    drop(scratch); // Proven cancellation disposes scratch, not spare ledger backing.
    assert_eq!(fixture.ports.counters().active_operation_bytes(), funded);
    ledger.pending_events.clear(); // The checkpoint event drain retains its capacity.
    assert_eq!(ledger.publication_backing_bytes(), Some(retained));
    assert_eq!(fixture.ports.counters().active_operation_bytes(), funded);
    let scratch = ledger
        .prepare_publication_backing(&fixture.owner, fixture.ceiling, 128, 64, key())
        .unwrap();
    drop(scratch); // Successful commit also disposes scratch without shrinking the owner.
    assert_eq!(fixture.ports.counters().active_operation_bytes(), funded);
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
    assert!(!fixture.ports.close().requires_inspection());
}

#[test]
fn uncertain_pending_scratch_keeps_charge_after_selected_ledger_disposal() {
    let fixture = fixture();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    let root_frame = ledger
        .prepare_publication_backing(&fixture.owner, fixture.ceiling, 128, 64, key())
        .unwrap();
    let pending = super::super::ReleaseCertificatePending {
        key: key(),
        root_frame,
        needed_records: 2,
        worst_case_encoded_bytes: 1024,
        effect_may_exist: true,
    };
    let funded = fixture.ports.counters().active_operation_bytes();
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), funded);
    assert!(pending.root_frame.bytes.capacity() > 0);
    drop(pending);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn failed_growth_funds_old_plus_prospective_capacity_before_changing_vector() {
    let fixture = fixture();
    let mut custody = None;
    let mut values = Vec::<u8>::new();
    let mut window =
        LiveBackingWindow::new(&fixture.owner, &mut custody, fixture.ceiling, 1000).unwrap();
    window.grow_vec(&mut values, 4).unwrap();
    values.extend_from_slice(&[1, 2, 3, 4]);
    drop(window);
    assert_eq!(values.capacity(), 4);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 1004);
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_536 - 1004 - 7).unwrap())
        .unwrap();
    let mut window =
        LiveBackingWindow::new(&fixture.owner, &mut custody, fixture.ceiling, 1004).unwrap();
    let Err(Denial::Resident(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause))) =
        window.grow_vec(&mut values, 4)
    else {
        panic!("eight prospective bytes cannot fit seven available bytes");
    };
    assert_eq!(cause.pressure().unwrap().requested(), 8);
    assert_eq!(values.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(values.capacity(), 4);
    drop(window);
    drop(collision);
    drop(values);
    drop(custody);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn original_numeric_ceiling_remains_independent_of_live_pool_headroom() {
    let fixture = fixture();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    let smaller = PhysicalRecoveryAllocationAdmission::new(fixture.ceiling.store_identity(), 128);
    assert!(matches!(
        ledger.prepare_publication_backing(&fixture.owner, smaller, 128, 0, key()),
        Err(Denial::Resident(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { admitted: 128, .. }
        ))
    ));
    assert_eq!(ledger.publication_backing_bytes(), Some(0));
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn foreign_runtime_or_stale_generation_cannot_grow_existing_charge() {
    let fixture = fixture();
    let mut custody = None;
    fixture
        .owner
        .fund(&mut custody, fixture.ceiling, 1000)
        .unwrap();
    let foreign = ReleasePublicationAllocationOwner::new(
        fixture.ports.clone(),
        fixture.ceiling,
        RuntimeIdentity::from_reopened(NonZeroU64::new(2).unwrap()),
        fixture.lifecycle.snapshot().generation,
        fixture.lifecycle.observation_state(),
        0,
    );
    assert!(matches!(
        foreign.fund(&mut custody, fixture.ceiling, 1001),
        Err(Denial::SelectedFactMismatch)
    ));
    fixture.lifecycle.begin_termination();
    assert!(matches!(
        fixture.owner.fund(&mut custody, fixture.ceiling, 1001),
        Err(Denial::SelectedFactMismatch)
    ));
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 1000);
    drop(custody);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}
