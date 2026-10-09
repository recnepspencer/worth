//! The same canonical walk crosses a real bounded native allocation owner.

use super::*;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;
use std::{mem::size_of, num::NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

#[test]
fn native_initial_pressure_precedes_read_and_funded_slices_survive_coordination() {
    let (_directory, _media, coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let events = ports.allocation_events();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frame, digest) = one_head();
    let limit = window.recovery_byte_limit();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(limit - 1).unwrap())
        .unwrap();
    let before = events.snapshot();
    let mut numeric = resident(&window, limit);
    let mut reads = 0;
    let denial = observe_with_read(
        &root,
        format(),
        1,
        digest,
        limit,
        &window,
        &mut numeric,
        |_, _, storage| {
            reads += 1;
            charged_frame(&frame, storage)
        },
    )
    .err()
    .unwrap();
    assert!(
        matches!(denial, Denial::Resident(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { admitted, .. }) if admitted == limit)
    );
    assert_eq!(reads, 0);
    assert_eq!(numeric.used(), 0);
    let after = events.snapshot();
    for dimension in [
        Dimension::OperationScope(Scope::Recovery),
        Dimension::OperationBytes,
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units()
        );
        assert_eq!(
            after.for_dimension(dimension).active_units(),
            before.for_dimension(dimension).active_units()
        );
    }
    drop(held);
    let observed = observe_with_read(
        &root,
        format(),
        1,
        digest,
        limit,
        &window,
        &mut numeric,
        |_, _, storage| charged_frame(&frame, storage),
    )
    .unwrap();
    let entries =
        observed.entries.capacity() as u64 * size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let slices = observed.slices.capacity() as u64 * size_of::<SelectedArtifactSlice>() as u64;
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        entries + slices
    );
    let funded = observed
        .into_funded_slices_with_resident(&mut numeric)
        .unwrap();
    assert_eq!(funded.charged_bytes(), slices);
    assert_eq!(numeric.used(), slices);
    assert_eq!(funded.owned_heap_bytes(), Some(slices));
    assert!(funded.matching_owner(&window));
    let (_foreign_directory, _foreign_media, foreign) = fixture::coordination();
    let foreign_window = PhysicalRecoveryReadAllocation::for_coordination(&foreign).unwrap();
    assert!(!funded.matching_owner(&foreign_window));
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        slices
    );
    assert_eq!(funded.slices().len(), 1);
    drop(funded);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    drop(ports);
    assert_eq!(
        events
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn branch_growth_denial_unwinds_all_implicit_buffers_then_same_owner_retries() {
    let (_directory, _media, coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let limit = window.recovery_byte_limit();
    let (root, frames, digest) = branched_heads();
    let mut numeric = resident(&window, limit);
    let mut held = None;
    let mut reads = 0;
    let denied = observe_with_read(
        &root,
        format(),
        2,
        digest,
        limit,
        &window,
        &mut numeric,
        |reference, _, storage| {
            reads += 1;
            // Exercise retained spare capacity honestly: the valid encoded
            // bytes use a full-page allocation, not a padded/altered frame.
            let mut frame = storage.reserve_vec::<u8>(format().page_size().bytes() as usize)?;
            frame.extend_from_slice(&frames[(reference.block() - 1) as usize]);
            let active = ports.counters().active_operation_bytes_for(Scope::Recovery);
            held = Some(
                ports
                    .begin_operation(Scope::Recovery, NonZeroU64::new(limit - active).unwrap())
                    .unwrap(),
            );
            Ok(frame)
        },
    );
    assert!(
        matches!(denied, Err(Denial::Resident(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { admitted, .. })) if admitted == limit)
    );
    assert_eq!(reads, 1);
    assert_eq!(
        numeric.used(),
        0,
        "frame, DFS, seen table and outputs unwound"
    );
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        held.as_ref().unwrap().bytes()
    );
    drop(held);
    let observed = observe_with_read(
        &root,
        format(),
        2,
        digest,
        limit,
        &window,
        &mut numeric,
        |reference, _, storage| charged_frame(&frames[(reference.block() - 1) as usize], storage),
    )
    .unwrap();
    assert_eq!(observed.entries(), &[head(1), head(2)]);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        numeric.used()
    );
    drop(observed);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn malformed_block_and_roster_mismatch_release_scratch_without_discard_callbacks() {
    let (_directory, _media, coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frame, digest) = one_head();
    for invalid_frame in [true, false] {
        let mut numeric = resident(&window, window.recovery_byte_limit());
        let bad_digest = if invalid_frame { digest } else { [0; 32] };
        let result = observe_with_read(
            &root,
            format(),
            1,
            bad_digest,
            window.recovery_byte_limit(),
            &window,
            &mut numeric,
            |_, _, storage| {
                let mut copied = charged_frame(&frame, storage)?;
                if invalid_frame {
                    copied[0] ^= 1;
                }
                Ok(copied)
            },
        );
        assert!(matches!(result, Err(Denial::CertificateRoster)));
        assert_eq!(numeric.used(), 0);
        assert_eq!(ports.counters().active_operation_bytes(), 0);
    }
}

#[test]
fn second_native_observation_preserves_first_charge_and_empty_keeps_owner_binding() {
    let (_directory, _media, coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let limit = window.recovery_byte_limit();
    let (root, frame, digest) = one_head();
    let mut numeric = resident(&window, limit);
    let first = observe_with_read(
        &root,
        format(),
        1,
        digest,
        limit,
        &window,
        &mut numeric,
        |_, _, storage| charged_frame(&frame, storage),
    )
    .unwrap();
    let first_bytes = ports.counters().active_operation_bytes_for(Scope::Recovery);
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(limit - first_bytes - 1).unwrap(),
        )
        .unwrap();
    let mut reads = 0;
    let denied = observe_with_read(
        &root,
        format(),
        1,
        digest,
        limit,
        &window,
        &mut numeric,
        |_, _, storage| {
            reads += 1;
            charged_frame(&frame, storage)
        },
    );
    assert!(matches!(
        denied,
        Err(Denial::Resident(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { .. }
        ))
    ));
    assert_eq!(reads, 0);
    assert_eq!(numeric.used(), first_bytes);
    assert_eq!(first.entries(), &[head(1)]);
    drop(held);
    let second = observe_with_read(
        &root,
        format(),
        1,
        digest,
        limit,
        &window,
        &mut numeric,
        |_, _, storage| charged_frame(&frame, storage),
    )
    .unwrap();
    assert!(first.same_bytes(&second));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        first_bytes + second.owned_heap_bytes().unwrap()
    );
    drop((first, second));
    assert_eq!(ports.counters().active_operation_bytes(), 0);
    let empty_root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .admit()
        .unwrap();
    let empty_digest = ReleaseCustodyHeadRosterDigestV1::new(None, 0).finish().1;
    let empty = observe_with_read(
        &empty_root,
        format(),
        0,
        empty_digest,
        limit,
        &window,
        &mut resident(&window, limit),
        |_, _, _| panic!("absent root has no read"),
    )
    .unwrap();
    let slices = empty.into_funded_slices().unwrap();
    assert_eq!(slices.charged_bytes(), 0);
    assert_eq!(slices.owned_heap_bytes(), Some(0));
    assert!(slices.matching_owner(&window));
}
