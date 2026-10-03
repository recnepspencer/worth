//! Independent capacity census and actual shared-pool lifetime probes.
#[path = "tests/command_reuse.rs"]
mod command_reuse;
use super::super::{backing::ReleasePublicationAllocationOwner, SelectedReleaseCustodyLedger};
use super::storage::*;
use crate::physical_runtime::{
    lifecycle::LifecycleCoordinator, PhysicalRecoveryAllocationAdmission, RuntimeIdentity,
};
use std::{
    num::{NonZeroU32, NonZeroU64},
    sync::Arc,
};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyLimits,
    PhysicalSpeculativeWorkKind as Speculation,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    CheckpointCertificateKind, PhysicalCheckpointIdentity, ReleaseCheckpointNoReleaseV1,
    ReleaseCustodyHeadKeyV1,
};

struct Fixture {
    owner: ReleasePublicationAllocationOwner,
    ports: crate::physical_runtime::record_serving::RecordFramePorts,
    ceiling: PhysicalRecoveryAllocationAdmission,
}
fn fixture() -> Fixture {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([118; 16]).unwrap(),
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
    );
    Fixture {
        owner,
        ports,
        ceiling,
    }
}
fn key() -> ReleaseCustodyHeadKeyV1 {
    ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap()
}
fn prepared(fixture: &Fixture) -> SelectedReleaseCustodyLedger {
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger
        .prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false)
        .unwrap();
    ledger
}
fn capsule(ledger: &SelectedReleaseCustodyLedger) -> Arc<SealedCheckpointStorage> {
    Arc::clone(
        ledger
            .checkpoint_backing
            .as_ref()
            .unwrap()
            .available
            .lock()
            .unwrap()
            .as_ref()
            .unwrap(),
    )
}
fn populate(ledger: &SelectedReleaseCustodyLedger, fixture: &Fixture) {
    let mut available = ledger
        .checkpoint_backing
        .as_ref()
        .unwrap()
        .available
        .lock()
        .unwrap();
    let capsule = Arc::get_mut(available.as_mut().unwrap()).unwrap();
    let mut preparation = capsule.take_preparation();
    preparation.observer.clear();
    let checkpoint =
        PhysicalCheckpointIdentity::new(fixture.ceiling.store_identity(), NonZeroU64::MIN);
    let marker =
        ReleaseCheckpointNoReleaseV1::new(checkpoint, 1, [1; 32], 0, [0; 32], [0; 32]).unwrap();
    preparation.observer.scratch = marker
        .encode_in_reserved(std::mem::take(&mut preparation.observer.scratch))
        .unwrap();
    preparation
        .observer
        .push(CheckpointCertificateKind::ReleasedDrop)
        .unwrap();
    capsule.put_preparation(preparation);
}
fn census(ledger: &SelectedReleaseCustodyLedger) -> u64 {
    let mut available = ledger
        .checkpoint_backing
        .as_ref()
        .unwrap()
        .available
        .lock()
        .unwrap();
    let capsule = Arc::get_mut(available.as_mut().unwrap()).unwrap();
    let mut preparation = capsule.take_preparation();
    let observer = &preparation.observer;
    let mut bytes = observer.bytes.capacity()
        + observer.records.capacity() * std::mem::size_of::<CertificateRange>()
        + observer.scratch.capacity()
        + observer.frame_scratch.capacity();
    let fold = preparation.fold.as_ref().unwrap();
    bytes += fold.heads.owned_heap_bytes().unwrap() as usize
        + fold.batches.capacity()
            * std::mem::size_of::<worth_store_physical_format::ReleaseCheckpointBatchV1>()
        + fold.scratch.capacity();
    bytes += preparation
        .command
        .as_mut()
        .unwrap()
        .bytes_mut()
        .unwrap()
        .capacity();
    bytes += std::mem::size_of::<SealedCheckpointStorage>()
        + std::mem::size_of::<ReusableCheckpointSlot>()
        + std::mem::size_of::<FundedCheckpointBufferPreparation>()
        + 6 * std::mem::size_of::<usize>();
    bytes += 2 * super::super::backing::SHARED_CUSTODY_BYTES as usize;
    capsule.put_preparation(preparation);
    bytes as u64
}

#[test]
fn actual_snapshot_buffers_and_all_shared_controls_have_live_pool_backing() {
    let fixture = fixture();
    let ledger = prepared(&fixture);
    assert!(fixture.ports.counters().active_operation_bytes() >= census(&ledger));
    populate(&ledger, &fixture);
    let observer = capsule(&ledger);
    let frame = observer.frame(0).unwrap();
    let certificate = observer.views().get(0).unwrap();
    let retained = fixture.ports.counters().active_operation_bytes();
    drop(ledger);
    drop(observer);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), retained);
    assert_eq!(
        worth_store_physical_format::decode_checkpoint_certificate(frame.bytes())
            .unwrap()
            .1,
        certificate.payload()
    );
    drop(frame);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), retained);
    drop(certificate);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn cancellation_reuses_exact_capsule_under_full_recovery_pressure() {
    let fixture = fixture();
    let mut ledger = prepared(&fixture);
    let first = capsule(&ledger);
    let pointer = Arc::as_ptr(&first);
    drop(first); // Cancellation has released all candidate observers.
    let retained = fixture.ports.counters().active_operation_bytes();
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_536 - retained).unwrap())
        .unwrap();
    ledger
        .prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false)
        .unwrap();
    populate(&ledger, &fixture);
    assert_eq!(Arc::as_ptr(&capsule(&ledger)), pointer);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 65_536);
    drop(collision);
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn held_snapshot_denies_next_preparation_before_any_new_backing_allocation() {
    let fixture = fixture();
    let mut ledger = prepared(&fixture);
    let held = capsule(&ledger);
    let retained = fixture.ports.counters().active_operation_bytes();
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_535 - retained).unwrap())
        .unwrap();
    assert!(matches!(
        ledger.prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false),
        Err(super::super::ReleaseCertificateCapacityDenial::Resident(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(_)
        ))
    ));
    assert_eq!(Arc::as_ptr(&capsule(&ledger)), Arc::as_ptr(&held));
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 65_535);
    drop(collision);
    ledger
        .prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false)
        .unwrap();
    assert_ne!(Arc::as_ptr(&capsule(&ledger)), Arc::as_ptr(&held));
    assert!(fixture.ports.counters().active_operation_bytes() >= 2 * retained);
    drop(held);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), retained);
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn committed_roster_owns_its_independent_fold_grant_after_snapshot_disposal() {
    let fixture = fixture();
    let ledger = prepared(&fixture);
    let observer = capsule(&ledger);
    let lease = observer.lease_fold().unwrap();
    let mut fold = lease.into_workspace();
    fold.heads
        .retain_fold_charge(Arc::clone(fold.custody.as_ref().unwrap()));
    let heads = std::mem::replace(
        &mut fold.heads,
        super::super::heads::SelectedReleaseHeadRoster::empty(),
    );
    let allocated_head_bytes = heads.owned_heap_bytes().unwrap();
    assert!(allocated_head_bytes > 0);
    drop(fold);
    drop(ledger);
    drop(observer);
    assert!(fixture.ports.counters().active_operation_bytes() >= allocated_head_bytes);
    drop(heads);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn unique_backing_growth_funds_old_and_prospective_capacity_before_reserve() {
    let fixture = fixture();
    let ledger = prepared(&fixture);
    let mut available = ledger
        .checkpoint_backing
        .as_ref()
        .unwrap()
        .available
        .lock()
        .unwrap();
    let capsule = Arc::get_mut(available.as_mut().unwrap()).unwrap();
    let mut preparation = capsule.take_preparation();
    let original = preparation.observer.bytes.capacity();
    let retained = fixture.ports.counters().active_operation_bytes();
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_535 - retained).unwrap())
        .unwrap();
    let result = preparation.grow(
        &fixture.owner,
        fixture.ceiling,
        20,
        21,
        original * 20,
        1024,
        20,
    );
    let Err(super::super::ReleaseCertificateCapacityDenial::Resident(
        crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause),
    )) = result
    else {
        panic!("real growth must encounter held Recovery pressure");
    };
    assert!(cause.pressure().unwrap().requested() >= original as u64 * 20);
    assert_eq!(preparation.observer.bytes.capacity(), original);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 65_535);
    capsule.put_preparation(preparation);
    drop(available);
    drop(collision);
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}
