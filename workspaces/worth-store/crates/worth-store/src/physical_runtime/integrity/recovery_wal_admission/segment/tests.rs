//! Genuine native pool evidence, with an independent actual-capacity census.

use super::*;
use crate::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryCoordination,
    PhysicalRecoveryRejoinResidentDenial,
};
use std::{alloc::Layout, num::NonZeroU64, sync::atomic::AtomicUsize};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_backend::{ReadGrant, UnchargedRead};
use worth_store_physical_format::wal_frame::{encode_wal_frame_v1, WalFrameV1EncodeRequest};
use worth_store_physical_integrity::{
    validate_wal_frame_prefix, UntrustedPhysicalArtifact, WalFrameIntegrityValidation,
};

fn fixture() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
    Vec<ObservedWalArtifact>,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    super::super::media_generation_tests::initialize(&root);
    let identity = worth_store_physical_format::WalSegmentIdentity::new(1, 1).unwrap();
    let mut bytes = Vec::new();
    for ordinal in 1..=3 {
        bytes.extend(encode_wal_frame_v1(
            WalFrameV1EncodeRequest::from_segment_identity(
                identity,
                ordinal,
                ordinal + 1,
                b"native-shared-wal",
                b"redo-payload",
            )
            .unwrap(),
        ));
    }
    let wal = root.join("families").join("wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join("segment-1-generation-1.wal"), bytes).unwrap();
    let (media, coordination) =
        super::super::media_generation_tests::recovery_media_and_coordination(&root);
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let observed = discovery
        .read_wal_artifacts(NonZeroU64::MIN, ReadGrant::ceiling_only())
        .observed()
        .unwrap();
    (directory, discovery.finish(), coordination, observed)
}

fn artifact() -> WalSegmentArtifactIdentity {
    WalSegmentArtifactIdentity::parse("segment-1-generation-1.wal").unwrap()
}

fn frame(
    coordination: &PhysicalRecoveryCoordination,
    observed: &ObservedWalArtifact,
    offset: usize,
) -> IntegrityAdmittedRecoveryWalFrame {
    let (validation, _) = validate_wal_frame_prefix(
        UntrustedPhysicalArtifact::from_bounded_bytes(&observed.bytes().unwrap()[offset..]),
        observed.store_identity(),
        crate::physical_runtime::recovery_wal::wal_frame_integrity_scope_identity(artifact()),
        offset as u64,
    );
    let WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("genuine frame rejected");
    };
    coordination
        .admit_recovery_wal_frame(
            observed,
            validated.scope(),
            validated.scope().byte_range(),
            validated,
        )
        .unwrap()
}

fn arc_layout<T>() -> u64 {
    Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}

#[test]
fn shared_frame_and_segment_keep_actual_capacity_charge_until_last_owner() {
    let (_directory, media, coordination, observed) = fixture();
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let mut builder = coordination
        .begin_recovery_wal_segment(&observed[0], artifact())
        .unwrap();
    let mut offset = 0;
    for _ in 0..3 {
        let admitted = frame(&coordination, &observed[0], offset);
        offset += admitted.encoded_byte_count() as usize;
        builder.push(admitted).unwrap();
    }
    let segment = builder.finish().unwrap();
    assert_eq!(segment.backing.frames.len(), 3);
    assert_eq!(segment.backing.frames.capacity(), 4);
    let frame_bytes = segment
        .frames()
        .iter()
        .map(|frame| {
            let actual = arc_layout::<super::super::AdmittedWalFrameData>()
                + frame.backing.source_name.capacity() as u64
                + frame.backing.encoded.capacity() as u64;
            assert_eq!(frame.owned_heap_bytes(), Some(actual));
            assert_eq!(frame.charged_bytes(), actual);
            actual
        })
        .sum::<u64>();
    let roster_bytes = arc_layout::<AdmittedWalSegmentData>()
        + (segment.backing.frames.capacity()
            * std::mem::size_of::<IntegrityAdmittedRecoveryWalFrame>()) as u64;
    assert_eq!(segment.charged_bytes(), roster_bytes);
    assert_eq!(segment.owned_heap_bytes(), Some(roster_bytes + frame_bytes));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        roster_bytes + frame_bytes
    );
    let shared = segment.clone();
    let escaped_frame = segment.frames()[0].clone();
    assert!(Arc::ptr_eq(&segment.backing, &shared.backing));
    let escaped_bytes = escaped_frame.charged_bytes();
    drop(segment);
    drop(coordination);
    drop(media);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        roster_bytes + frame_bytes
    );
    assert_eq!(escaped_frame.payload(), b"redo-payload");
    drop(shared);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        escaped_bytes
    );
    drop(escaped_frame);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(ports);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn first_frame_native_pressure_denies_before_admission_and_retries_exactly() {
    let (_directory, _media, coordination, observed) = fixture();
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let limit = owner.admitted_policy().operation_bytes();
    let held = ports
        .begin_operation(Scope::Maintenance, NonZeroU64::new(limit - 1).unwrap())
        .unwrap();
    let before = ports.allocation_events().snapshot();
    let (validation, _) = validate_wal_frame_prefix(
        UntrustedPhysicalArtifact::from_bounded_bytes(observed[0].bytes().unwrap()),
        observed[0].store_identity(),
        crate::physical_runtime::recovery_wal::wal_frame_integrity_scope_identity(artifact()),
        0,
    );
    let WalFrameIntegrityValidation::Intact(validated) = validation else {
        panic!("fixture intact");
    };
    let denial = coordination
        .admit_recovery_wal_frame(
            &observed[0],
            validated.scope(),
            validated.scope().byte_range(),
            validated,
        )
        .unwrap_err();
    let Denial::Allocation(RecoveryWalAllocationDenial::Backing {
        cause: PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure),
        ..
    }) = denial
    else {
        panic!("native aggregate pressure must remain typed");
    };
    let pressure = failure.pressure().unwrap();
    assert_eq!(pressure.dimension(), Dimension::OperationBytes);
    assert_eq!(pressure.admitted(), limit - 1);
    assert!(!pressure.effect_may_have_started());
    let after = ports.allocation_events().snapshot();
    for dimension in [
        Dimension::OperationBytes,
        Dimension::OperationScope(Scope::Recovery),
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            before.for_dimension(dimension).admissions(),
            after.for_dimension(dimension).admissions()
        );
        assert_eq!(
            before.for_dimension(dimension).admitted_units(),
            after.for_dimension(dimension).admitted_units()
        );
    }
    drop(held);
    let admitted = frame(&coordination, &observed[0], 0);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        admitted.charged_bytes()
    );
    drop(admitted);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn roster_growth_native_denial_preserves_old_capacity_then_retry_and_cancel_release() {
    let (_directory, _media, coordination, observed) = fixture();
    let (owner, original, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let first = frame(&coordination, &observed[0], 0);
    let second = frame(
        &coordination,
        &observed[0],
        first.encoded_byte_count() as usize,
    );
    let second_retry = second.clone();
    let mut builder = coordination
        .begin_recovery_wal_segment(&observed[0], artifact())
        .unwrap();
    builder.push(first).unwrap();
    let old_capacity = builder.frames.capacity();
    let old_charge = builder.grant.as_ref().unwrap().bytes();
    let current = ports.counters().active_operation_bytes_for(Scope::Recovery);
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(original.byte_limit() - current - 1).unwrap(),
        )
        .unwrap();
    let before = ports.allocation_events().snapshot();
    let denial = builder.push(second).unwrap_err();
    let Denial::Allocation(RecoveryWalAllocationDenial::Backing {
        requested,
        cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { admitted, .. },
    }) = denial
    else {
        panic!("original live ceiling must deny native roster growth");
    };
    assert!(requested > old_charge);
    assert_eq!(admitted, original.byte_limit());
    assert_eq!(builder.frames.capacity(), old_capacity);
    assert_eq!(builder.len(), 1);
    assert_eq!(builder.grant.as_ref().unwrap().bytes(), old_charge);
    let after = ports.allocation_events().snapshot();
    for dimension in [
        Dimension::OperationBytes,
        Dimension::OperationScope(Scope::Recovery),
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            before.for_dimension(dimension).admissions(),
            after.for_dimension(dimension).admissions()
        );
        assert_eq!(
            before.for_dimension(dimension).admitted_units(),
            after.for_dimension(dimension).admitted_units()
        );
    }
    drop(held);
    builder.push(second_retry).unwrap();
    assert_eq!(builder.len(), 2);
    drop(builder);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn oversized_allocator_result_is_disposed_and_builder_cannot_reuse_it() {
    let (_directory, _media, coordination, observed) = fixture();
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let admitted = frame(&coordination, &observed[0], 0);
    let retained = admitted.clone();
    let retained_charge = retained.charged_bytes();
    let mut builder = coordination
        .begin_recovery_wal_segment(&observed[0], artifact())
        .unwrap();
    builder.push(admitted).unwrap();
    let permitted = builder.frames.capacity();
    let requested = (permitted * std::mem::size_of::<IntegrityAdmittedRecoveryWalFrame>()) as u64;
    // Inject an actual wider Vec into the private post-allocator boundary.
    // This tests quarantine, not a claim that the injected Vec was funded.
    builder.frames.reserve_exact(permitted + 1);
    let actual = (builder.frames.capacity()
        * std::mem::size_of::<IntegrityAdmittedRecoveryWalFrame>()) as u64;
    assert!(actual > requested);
    let cause = RecoveryWalAllocationDenial::AllocatorExceededReservation { requested, actual };
    assert_eq!(
        builder.accept_allocated_roster(requested),
        Err(Denial::Allocation(cause.clone()))
    );
    assert_eq!(builder.frames.capacity(), 0);
    assert!(builder.grant.is_none());
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        retained_charge
    );
    assert_eq!(
        builder.push(retained.clone()),
        Err(Denial::Allocation(cause.clone()))
    );
    assert!(matches!(builder.finish(), Err(Denial::Allocation(found)) if found == cause));
    assert_eq!(retained.payload(), b"redo-payload");
    drop(retained);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
