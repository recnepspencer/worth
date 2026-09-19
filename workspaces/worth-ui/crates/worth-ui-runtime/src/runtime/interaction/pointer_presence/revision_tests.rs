use super::*;

#[test]
fn exhausted_observation_revision_preserves_primary_and_pointer_records() {
    let session = crate::runtime::tests::active_application_session_test_support::source_backed_component_session();
    let generation = session.active_generation_identity();
    let mut owner = UiPointerPresenceOwner::new(UiPointerPresenceCapacity::for_test(2));
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );
    let target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = worth_ui_host_contract::UiMountedNodeReceiptIdentity::mint_unbound().unwrap();
    owner
        .record_pointer_target(
            UiHostPointerIdentity::new(1),
            UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(1),
            UiHostSurfacePosition::viewport_logical(0, 0),
            presentation,
            Some((surface, binding, target, receipt)),
            &generation,
        )
        .unwrap();
    // Unit-level counter fault injection; this test makes no presentation-admission claim.
    owner.revision = u64::MAX;
    let predecessor = owner.appearance_snapshot();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        owner.record_pointer_target(
            UiHostPointerIdentity::new(2),
            UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(2),
            UiHostSurfacePosition::viewport_logical(0, 0),
            presentation,
            Some((surface, binding, target, receipt)),
            &generation,
        )
    }));
    assert!(failed.is_err());
    assert_eq!(owner.appearance_snapshot(), predecessor);
    assert_eq!(owner.pointer_count(), 1);
    let _ = session.shutdown();
}
