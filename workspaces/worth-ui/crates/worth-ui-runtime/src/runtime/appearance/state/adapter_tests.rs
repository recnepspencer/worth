use super::{
    UiAppearanceCoherentBasis, UiAppearanceOwnerSnapshot, UiAppearanceSelectionSelector,
    UiAppearanceStateAdapterDenial, UiAppearanceStateAxisDemand, UiAppearanceStateConsumer,
};

use worth_ui_dsl::{UiAppearanceAxisClass, UiAppearanceStateAxis};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostObservationSequence, UiHostPointerIdentity,
    UiHostSurfacePosition, UiMountIncarnation, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

pub(super) struct Fixture {
    pub(super) snapshot: UiAppearanceOwnerSnapshot,
    pub(super) basis: UiAppearanceCoherentBasis,
    pub(super) selection: UiAppearanceSelectionSelector,
}

#[test]
fn all_six_adapters_consume_one_sealed_owner_snapshot() {
    let fixture = fixture();
    assert_eq!(
        super::operability::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::OperabilityReady
    );
    assert_eq!(
        super::focus::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::FocusUnfocused
    );
    assert_eq!(
        super::validation::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::ValidationValid
    );
    assert_eq!(
        super::selection::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::SelectedAnchorCursor
    );
    assert_eq!(
        super::hover::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::Hovered
    );
    assert_eq!(
        super::pressed::adapt(&fixture.snapshot, &fixture.basis)
            .unwrap()
            .class(),
        UiAppearanceAxisClass::PressedIdle
    );
}

#[test]
fn adapters_deny_missing_owner_and_missing_selection_source() {
    let fixture = fixture();
    let generation = fixture.snapshot.generation().clone();
    let authority = crate::runtime::observation::UiObservationTurnCloseAuthority::for_test();
    let missing = UiAppearanceOwnerSnapshot::seal_at_turn_close(
        &authority,
        crate::runtime::observation::UiObservationTurnIdentity::for_test(2),
        fixture.snapshot.session(),
        fixture.snapshot.source_basis(),
        generation,
        demand(),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        super::hover::adapt(&missing, &fixture.basis),
        Err(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Hover
        ))
    );
    assert_eq!(
        super::operability::adapt(&missing, &fixture.basis),
        Err(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Operability
        ))
    );
    assert_eq!(
        super::focus::adapt(&missing, &fixture.basis),
        Err(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Focus
        ))
    );
    assert_eq!(
        super::validation::adapt(&missing, &fixture.basis),
        Err(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Validation
        ))
    );
    assert_eq!(
        super::pressed::adapt(&missing, &fixture.basis),
        Err(UiAppearanceStateAdapterDenial::MissingOwner(
            UiAppearanceStateAxis::Pressed
        ))
    );
    let selectionless_basis = UiAppearanceCoherentBasis::for_test(
        &fixture.snapshot,
        UiAppearanceStateConsumer::all_axes_for_test(fixture.basis.graph_node()),
        fixture.basis.mounted_instance(),
        fixture.basis.incarnation(),
        fixture.basis.node_receipt(),
        fixture.basis.surface(),
        fixture.basis.presentation(),
        None,
        Some("select".into()),
    );
    assert_eq!(
        super::selection::adapt(&fixture.snapshot, &selectionless_basis),
        Err(UiAppearanceStateAdapterDenial::MissingSource(
            UiAppearanceStateAxis::Selection
        ))
    );
}

#[test]
fn selection_adapter_rejects_stale_foreign_and_ambiguous_snapshot_keys() {
    let fixture = fixture();
    let owner = fixture.selection.owner();
    let key = fixture.selection.key();
    let stale = UiAppearanceCoherentBasis::for_test(
        &fixture.snapshot,
        UiAppearanceStateConsumer::all_axes_for_test(fixture.basis.graph_node()),
        fixture.basis.mounted_instance(),
        fixture.basis.incarnation(),
        fixture.basis.node_receipt(),
        fixture.basis.surface(),
        fixture.basis.presentation(),
        Some(UiAppearanceSelectionSelector::new(
            owner,
            key,
            crate::runtime::selection::UiSelectionOwnerIncarnation::new(9).unwrap(),
        )),
        Some("select".into()),
    );
    assert_eq!(
        super::selection::adapt(&fixture.snapshot, &stale),
        Err(UiAppearanceStateAdapterDenial::StaleSource(
            UiAppearanceStateAxis::Selection
        ))
    );

    let foreign = UiAppearanceCoherentBasis::for_test(
        &fixture.snapshot,
        UiAppearanceStateConsumer::all_axes_for_test(fixture.basis.graph_node()),
        fixture.basis.mounted_instance(),
        fixture.basis.incarnation(),
        fixture.basis.node_receipt(),
        fixture.basis.surface(),
        fixture.basis.presentation(),
        Some(UiAppearanceSelectionSelector::new(
            owner,
            crate::runtime::selection::UiSelectionStableKey::new(
                crate::runtime::UiApplicationItemKey::new(
                    crate::runtime::UiApplicationItemKeyFamily::new(
                        core::num::NonZeroU64::new(99).unwrap(),
                    ),
                    core::num::NonZeroU64::new(1).unwrap(),
                ),
            ),
            fixture.selection.incarnation(),
        )),
        Some("select".into()),
    );
    assert_eq!(
        super::selection::adapt(&fixture.snapshot, &foreign),
        Err(UiAppearanceStateAdapterDenial::ForeignSource(
            UiAppearanceStateAxis::Selection
        ))
    );

    let ambiguous = selection_with_ambiguous_owner(&fixture);
    let basis = UiAppearanceCoherentBasis::for_test(
        &ambiguous,
        UiAppearanceStateConsumer::all_axes_for_test(fixture.basis.graph_node()),
        fixture.basis.mounted_instance(),
        fixture.basis.incarnation(),
        fixture.basis.node_receipt(),
        fixture.basis.surface(),
        fixture.basis.presentation(),
        Some(fixture.selection),
        Some("select".into()),
    );
    assert_eq!(
        super::selection::adapt(&ambiguous, &basis),
        Err(UiAppearanceStateAdapterDenial::AmbiguousSource(
            UiAppearanceStateAxis::Selection
        ))
    );
}

pub(super) fn fixture() -> Fixture {
    let session = crate::runtime::tests::active_application_session_test_support::
        source_backed_component_session();
    let generation = session.active_generation_identity();
    let session_identity = session.session_identity();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let graph_node = crate::graph::UiGraphNodeIdentity::new(71);
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let receipt = UiMountedNodeReceiptIdentity::mint_unbound().unwrap();
    let incarnation = UiMountIncarnation::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );

    let selection_owner = crate::runtime::selection::UiSelectionOwnerIdentity::new(
        surface,
        graph_node,
        crate::runtime::UiApplicationItemKeyFamily::new(core::num::NonZeroU64::new(3).unwrap()),
    );
    let selection_key = crate::runtime::selection::UiSelectionStableKey::new(
        crate::runtime::UiApplicationItemKey::new(
            selection_owner.key_family(),
            core::num::NonZeroU64::new(1).unwrap(),
        ),
    );
    let selection_incarnation =
        crate::runtime::selection::UiSelectionOwnerIncarnation::new(1).unwrap();
    let mut selection_state =
        crate::runtime::selection::UiSelectionRuntimeState::new_session_restore_candidate();
    selection_state
        .synchronize(
            crate::runtime::selection::UiSelectionRegistration::new(
                selection_owner,
                selection_incarnation,
                crate::runtime::selection::UiSelectionPolicy::Single,
                vec![selection_key],
                crate::runtime::selection::UiSelectionCatalogPosture::Complete,
            )
            .unwrap(),
        )
        .unwrap();
    selection_state
        .apply(
            selection_owner,
            selection_incarnation,
            crate::runtime::selection::UiSelectionRequest::SelectSingle(selection_key),
        )
        .unwrap();

    let mut pointer = crate::runtime::interaction::UiPointerPresenceOwner::new(
        crate::runtime::interaction::UiPointerPresenceCapacity::for_test(8),
    );
    pointer
        .record_pointer_target(
            UiHostPointerIdentity::new(1),
            crate::runtime::interaction::UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(1),
            UiHostSurfacePosition::viewport_logical(10, 10),
            presentation,
            Some((surface, binding, instance, receipt)),
            &generation,
        )
        .unwrap();
    let operability = crate::runtime::intent::UiIntentOperabilityStandingFactSnapshot::seal(
        1,
        vec![
            crate::runtime::intent::UiIntentOperabilityStandingFact::for_test(
                graph_node, instance, receipt, "select", 1,
            ),
        ],
    );
    let validation = crate::runtime::intent::UiValidationAppearanceFactSnapshot::for_test(
        graph_node,
        instance,
        receipt,
        crate::runtime::intent::UiValidationAppearanceClass::Valid,
    );
    let authority = crate::runtime::observation::UiObservationTurnCloseAuthority::for_test();
    let snapshot = UiAppearanceOwnerSnapshot::seal_at_turn_close(
        &authority,
        crate::runtime::observation::UiObservationTurnIdentity::for_test(1),
        session_identity,
        1,
        generation,
        demand(),
        Some(
            crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate()
                .appearance_posture(),
        ),
        Some(selection_state.appearance_owner_snapshot()),
        Some(operability),
        Some(validation),
        Some(pointer.appearance_snapshot()),
        Some(
            crate::runtime::interaction::gesture::UiPointerGestureRuntimeState::new(true)
                .appearance_snapshot(),
        ),
    );
    let selection =
        UiAppearanceSelectionSelector::new(selection_owner, selection_key, selection_incarnation);
    let basis = UiAppearanceCoherentBasis::for_test(
        &snapshot,
        UiAppearanceStateConsumer::all_axes_for_test(graph_node),
        instance,
        incarnation,
        receipt,
        surface,
        Some(presentation),
        Some(selection),
        Some("select".into()),
    );
    let _ = session.shutdown();
    Fixture {
        snapshot,
        basis,
        selection,
    }
}

fn selection_with_ambiguous_owner(fixture: &Fixture) -> UiAppearanceOwnerSnapshot {
    let owner = fixture.selection.owner();
    let incarnation = fixture.selection.incarnation();
    let second_owner = crate::runtime::selection::UiSelectionOwnerIdentity::new(
        owner.semantic_surface(),
        owner.graph_node(),
        crate::runtime::UiApplicationItemKeyFamily::new(core::num::NonZeroU64::new(4).unwrap()),
    );
    let second_key = crate::runtime::selection::UiSelectionStableKey::new(
        crate::runtime::UiApplicationItemKey::new(
            second_owner.key_family(),
            core::num::NonZeroU64::new(2).unwrap(),
        ),
    );
    let mut state =
        crate::runtime::selection::UiSelectionRuntimeState::new_session_restore_candidate();
    for (owner, key) in [(owner, fixture.selection.key()), (second_owner, second_key)] {
        state
            .synchronize(
                crate::runtime::selection::UiSelectionRegistration::new(
                    owner,
                    incarnation,
                    crate::runtime::selection::UiSelectionPolicy::Single,
                    vec![key],
                    crate::runtime::selection::UiSelectionCatalogPosture::Complete,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let authority = crate::runtime::observation::UiObservationTurnCloseAuthority::for_test();
    UiAppearanceOwnerSnapshot::seal_at_turn_close(
        &authority,
        fixture.snapshot.turn(),
        fixture.snapshot.session(),
        fixture.snapshot.source_basis(),
        fixture.snapshot.generation().clone(),
        demand(),
        fixture.snapshot.focus().copied(),
        Some(state.appearance_owner_snapshot()),
        fixture.snapshot.operability().cloned(),
        fixture.snapshot.validation().cloned(),
        fixture.snapshot.pointer_presence().cloned(),
        fixture.snapshot.pressed().cloned(),
    )
}

fn demand() -> UiAppearanceStateAxisDemand {
    let mut demand = UiAppearanceStateAxisDemand::default();
    for axis in [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
        UiAppearanceStateAxis::Selection,
        UiAppearanceStateAxis::Hover,
        UiAppearanceStateAxis::Pressed,
    ] {
        demand.include(axis);
    }
    demand
}
