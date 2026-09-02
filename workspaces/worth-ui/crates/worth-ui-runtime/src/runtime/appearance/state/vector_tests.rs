use super::{
    UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceStateVector,
    UiAppearanceStateVectorDenial,
};

#[test]
fn vector_seals_all_six_adapter_products_from_the_sealed_snapshot() {
    let fixture = super::adapter_tests::fixture();
    let vector = UiAppearanceStateVector::seal(&fixture.snapshot, &fixture.basis)
        .expect("one coherent owner snapshot should seal one vector");
    assert_eq!(vector.basis(), &fixture.basis);
    assert!(vector.operability().is_some());
    assert!(vector.focus().is_some());
    assert!(vector.validation().is_some());
    assert!(vector.selection().is_some());
    assert!(vector.hover().is_some());
    assert!(vector.pressed().is_some());
}

#[test]
fn vector_rejects_turn_session_source_and_generation_mismatches() {
    let fixture = super::adapter_tests::fixture();
    let foreign_session =
        crate::runtime::tests::active_application_session_test_support::source_backed_component_session();
    let foreign_generation = foreign_session.active_generation_identity();
    let _ = foreign_session.shutdown();
    let cases = [
        UiAppearanceOwnerSnapshot::seal_at_turn_close(
            &crate::runtime::observation::UiObservationTurnCloseAuthority::for_test(),
            crate::runtime::observation::UiObservationTurnIdentity::for_test(99),
            fixture.snapshot.session(),
            fixture.snapshot.source_basis(),
            fixture.snapshot.generation().clone(),
            fixture.snapshot.demand(),
            fixture.snapshot.focus().copied(),
            fixture.snapshot.selection().cloned(),
            fixture.snapshot.operability().cloned(),
            fixture.snapshot.validation().cloned(),
            fixture.snapshot.pointer_presence().cloned(),
            fixture.snapshot.pressed().cloned(),
        ),
        UiAppearanceOwnerSnapshot::seal_at_turn_close(
            &crate::runtime::observation::UiObservationTurnCloseAuthority::for_test(),
            fixture.snapshot.turn(),
            crate::lifecycle::WorthUiActiveApplicationSessionIdentity::from_host_session_value(
                fixture.snapshot.session().as_u64() + 100,
            ),
            fixture.snapshot.source_basis(),
            fixture.snapshot.generation().clone(),
            fixture.snapshot.demand(),
            fixture.snapshot.focus().copied(),
            fixture.snapshot.selection().cloned(),
            fixture.snapshot.operability().cloned(),
            fixture.snapshot.validation().cloned(),
            fixture.snapshot.pointer_presence().cloned(),
            fixture.snapshot.pressed().cloned(),
        ),
        UiAppearanceOwnerSnapshot::seal_at_turn_close(
            &crate::runtime::observation::UiObservationTurnCloseAuthority::for_test(),
            fixture.snapshot.turn(),
            fixture.snapshot.session(),
            fixture.snapshot.source_basis() + 1,
            fixture.snapshot.generation().clone(),
            fixture.snapshot.demand(),
            fixture.snapshot.focus().copied(),
            fixture.snapshot.selection().cloned(),
            fixture.snapshot.operability().cloned(),
            fixture.snapshot.validation().cloned(),
            fixture.snapshot.pointer_presence().cloned(),
            fixture.snapshot.pressed().cloned(),
        ),
        UiAppearanceOwnerSnapshot::seal_at_turn_close(
            &crate::runtime::observation::UiObservationTurnCloseAuthority::for_test(),
            fixture.snapshot.turn(),
            fixture.snapshot.session(),
            fixture.snapshot.source_basis(),
            foreign_generation,
            fixture.snapshot.demand(),
            fixture.snapshot.focus().copied(),
            fixture.snapshot.selection().cloned(),
            fixture.snapshot.operability().cloned(),
            fixture.snapshot.validation().cloned(),
            fixture.snapshot.pointer_presence().cloned(),
            fixture.snapshot.pressed().cloned(),
        ),
    ];
    for changed in cases {
        assert_eq!(
            UiAppearanceStateVector::seal(&changed, &fixture.basis),
            Err(UiAppearanceStateVectorDenial::SnapshotChanged)
        );
    }
}

#[test]
fn vector_rejects_a_presentation_that_disagrees_with_owner_postures() {
    let fixture = super::adapter_tests::fixture();
    let current = fixture.basis.presentation().expect("fixture presentation");
    let changed = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        current.frame(),
        current.binding(),
        current.epoch(),
    );
    let basis = UiAppearanceCoherentBasis::for_test(
        &fixture.snapshot,
        fixture.basis.consumer().clone(),
        fixture.basis.mounted_instance(),
        fixture.basis.incarnation(),
        fixture.basis.node_receipt(),
        fixture.basis.surface(),
        Some(changed),
        fixture.basis.selection(),
        fixture.basis.operability_route().map(Box::<str>::from),
    );
    assert_eq!(
        UiAppearanceStateVector::seal(&fixture.snapshot, &basis),
        Err(UiAppearanceStateVectorDenial::PresentationChanged)
    );
}
