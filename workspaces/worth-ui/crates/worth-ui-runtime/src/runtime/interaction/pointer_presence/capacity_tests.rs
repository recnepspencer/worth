use super::super::UiPointerPresenceAdmissionDenial;
use super::*;

#[test]
fn pointer_capacity_denies_without_eviction_and_recovers_after_retirement() {
    let mut owner = UiPointerPresenceOwner::new(UiPointerPresenceCapacity::for_test(1));
    let generation = crate::runtime::tests::active_application_session_test_support::
        source_backed_component_session();
    let generation_identity = generation.active_generation_identity();
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let presentation = presentation_basis(binding);
    let target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = worth_ui_host_contract::UiMountedNodeReceiptIdentity::mint_unbound().unwrap();
    let position = UiHostSurfacePosition::viewport_logical(4, 8);
    let first = UiHostPointerIdentity::new(1);
    let second = UiHostPointerIdentity::new(2);

    owner
        .record_pointer_target(
            first,
            UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(1),
            position,
            presentation,
            Some((surface, binding, target, receipt)),
            &generation_identity,
        )
        .unwrap();
    assert_eq!(owner.pointer_count(), 1);
    assert_eq!(owner.primary_count(), 1);
    assert_eq!(
        owner.record_pointer_target(
            second,
            UiPrimaryPointerKind::Stylus,
            UiHostObservationSequence::new(2),
            position,
            presentation,
            Some((surface, binding, target, receipt)),
            &generation_identity,
        ),
        Err(UiPointerPresenceAdmissionDenial::CapacityExceeded {
            pointer: second,
            limit: 1,
        })
    );
    assert_eq!(owner.pointer_count(), 1);
    assert!(owner.retire_pointer(first));
    assert_eq!(owner.pointer_count(), 0);
    assert_eq!(owner.primary_count(), 0);
    owner
        .record_pointer_target(
            second,
            UiPrimaryPointerKind::Stylus,
            UiHostObservationSequence::new(3),
            position,
            presentation,
            Some((surface, binding, target, receipt)),
            &generation_identity,
        )
        .unwrap();
    assert_eq!(owner.pointer_count(), 1);
    let _ = generation.shutdown();
}

#[test]
fn cancel_all_is_an_exact_pointer_and_primary_shutdown_census() {
    let mut owner = UiPointerPresenceOwner::new(UiPointerPresenceCapacity::for_test(2));
    let generation = crate::runtime::tests::active_application_session_test_support::
        source_backed_component_session();
    let generation_identity = generation.active_generation_identity();
    let first_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let second_surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first_binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let second_binding =
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let position = UiHostSurfacePosition::viewport_logical(4, 8);
    owner
        .record_pointer_target(
            UiHostPointerIdentity::new(3),
            UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(1),
            position,
            presentation_basis(first_binding),
            Some((
                first_surface,
                first_binding,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                worth_ui_host_contract::UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
            )),
            &generation_identity,
        )
        .unwrap();
    owner
        .record_pointer_target(
            UiHostPointerIdentity::new(4),
            UiPrimaryPointerKind::Touch,
            UiHostObservationSequence::new(2),
            position,
            presentation_basis(second_binding),
            Some((
                second_surface,
                second_binding,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                worth_ui_host_contract::UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
            )),
            &generation_identity,
        )
        .unwrap();
    assert_eq!(owner.pointer_count(), 2);
    assert_eq!(owner.primary_count(), 1);
    owner.cancel_all();
    assert_eq!(owner.pointer_count(), 0);
    assert_eq!(owner.primary_count(), 0);
    assert!(owner.appearance_snapshot().postures().is_empty());
    let _ = generation.shutdown();
}

fn presentation_basis(
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
) -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}
