use super::super::state_test_fixture::{incarnation, item_key_family, key, owner, registration};
use super::super::{
    UiSelectionAppearanceClass, UiSelectionAppearancePostureDenial, UiSelectionPolicy,
    UiSelectionRuntimeState,
};

#[test]
fn appearance_selection_export_is_keyed_and_rejects_reincarnation() {
    let owner = owner();
    let selected = key(1);
    let absent = key(2);
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            vec![selected, absent],
            super::super::UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            super::super::UiSelectionRequest::SelectSingle(selected),
        )
        .unwrap();
    let snapshot = state.appearance_owner_snapshot();

    let selected_posture = snapshot
        .posture_for(owner, selected, incarnation())
        .unwrap();
    assert_eq!(
        selected_posture.class(),
        UiSelectionAppearanceClass::SelectedAnchorCursor
    );
    assert_eq!(selected_posture.source_bits(), (true, true, true));
    let absent_posture = snapshot.posture_for(owner, absent, incarnation()).unwrap();
    assert_eq!(
        absent_posture.class(),
        UiSelectionAppearanceClass::Unselected
    );
    assert_eq!(absent_posture.source_bits(), (false, false, false));
    assert_eq!(
        snapshot.posture_for(
            owner,
            selected,
            super::super::UiSelectionOwnerIncarnation::new(8).unwrap(),
        ),
        Err(UiSelectionAppearancePostureDenial::StaleOwnerIncarnation)
    );
    let foreign = super::super::UiSelectionOwnerIdentity::new(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        crate::graph::UiGraphNodeIdentity::new(72),
        item_key_family(),
    );
    assert_eq!(
        snapshot.posture_for(foreign, selected, incarnation()),
        Err(UiSelectionAppearancePostureDenial::UnknownOwner)
    );
    let foreign_key =
        super::super::UiSelectionStableKey::new(crate::runtime::UiApplicationItemKey::new(
            crate::runtime::UiApplicationItemKeyFamily::new(
                core::num::NonZeroU64::new(99).unwrap(),
            ),
            core::num::NonZeroU64::new(1).unwrap(),
        ));
    assert_eq!(
        snapshot.posture_for(owner, foreign_key, incarnation()),
        Err(UiSelectionAppearancePostureDenial::ForeignItemKeyFamily)
    );
}

#[test]
fn appearance_selection_export_retains_partial_catalog_posture_keys() {
    let owner = owner();
    let retained = key(1);
    let replacement = key(2);
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            vec![retained],
            super::super::UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            super::super::UiSelectionRequest::SelectSingle(retained),
        )
        .unwrap();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            vec![replacement],
            super::super::UiSelectionCatalogPosture::Partial,
        ))
        .unwrap();

    let posture = state
        .appearance_owner_snapshot()
        .posture_for(owner, retained, incarnation())
        .unwrap();
    assert_eq!(
        posture.class(),
        UiSelectionAppearanceClass::SelectedAnchorCursor
    );
    assert_eq!(posture.source_bits(), (true, true, true));
}
