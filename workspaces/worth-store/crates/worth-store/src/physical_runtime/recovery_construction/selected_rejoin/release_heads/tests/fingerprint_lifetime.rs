//! Fingerprint mode moves and rejected merges never detach native slice custody.

use super::*;
use crate::physical_runtime::recovery_construction::SelectedControlMediaFingerprint;
use worth_store_buffer_pool::PhysicalOperationAllocationScope as Scope;

fn funded(
    window: &PhysicalRecoveryReadAllocation<'_>,
    numeric: &mut StoreRejoinResidentLedger,
) -> FundedHeadSlices {
    let (root, frame, digest) = one_head();
    observe_with_read(
        &root,
        format(),
        1,
        digest,
        window.recovery_byte_limit(),
        window,
        numeric,
        |_, _, storage| charged_frame(&frame, storage),
    )
    .unwrap()
    .into_funded_slices_with_resident(numeric)
    .unwrap()
}

#[test]
fn rejected_funded_mode_merge_disposes_donor_without_changing_destination_charge() {
    for with_numeric in [false, true] {
        let (_directory, _media, coordination) = fixture::coordination();
        let ports = coordination
            .sampling_allocation_basis()
            .unwrap()
            .0
            .ports()
            .clone();
        let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
        let mut numeric = resident(&window, window.recovery_byte_limit());
        let first = funded(&window, &mut numeric);
        let first_bytes = first.charged_bytes();
        let mut destination = SelectedControlMediaFingerprint::selected_heads(first);
        let second = funded(&window, &mut numeric);
        let second_bytes = second.charged_bytes();
        let donor = SelectedControlMediaFingerprint::selected_heads(second);
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            first_bytes + second_bytes
        );
        let result = if with_numeric {
            destination.extend_with_resident(donor, &mut numeric)
        } else {
            destination.extend(donor)
        };
        assert!(matches!(result, Err(Denial::CertificateRoster)));
        assert!(destination.has_selected_head_walk());
        assert_eq!(
            destination.independently_funded_heap_bytes(),
            Some(first_bytes)
        );
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            first_bytes
        );
        drop(destination);
        assert_eq!(ports.counters().active_operation_bytes(), 0);
    }
}

#[test]
fn pending_mode_keeps_three_actual_owners_while_raw_merge_moves_only_the_envelope() {
    let (_directory, _media, coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut numeric = resident(&window, window.recovery_byte_limit());
    // Three actual canonical observations test owner mechanics, not pending-WAL
    // authority. Production's pending custody must still join its distinct roots.
    let checkpoint = funded(&window, &mut numeric);
    let pre_pending = funded(&window, &mut numeric);
    let effective = funded(&window, &mut numeric);
    let bytes =
        checkpoint.charged_bytes() + pre_pending.charged_bytes() + effective.charged_bytes();
    let pending =
        SelectedControlMediaFingerprint::pending_heads(checkpoint, pre_pending, effective);
    assert!(pending.has_pending_head_walks());
    assert_eq!(pending.owned_heap_bytes(), Some(bytes));
    let mut destination = SelectedControlMediaFingerprint::observed(Vec::new());
    destination
        .extend_with_resident(pending, &mut numeric)
        .unwrap();
    assert!(destination.has_pending_head_walks());
    assert_eq!(destination.independently_funded_heap_bytes(), Some(bytes));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        bytes
    );
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        bytes
    );
    drop(destination);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
