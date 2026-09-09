use super::state_test_fixture::incarnation;
use super::*;

#[test]
fn suspended_projection_catalog_reconciles_without_losing_stable_selection() {
    let family = crate::runtime::UiApplicationItemKeyFamily::from_projection_input(
        worth_ui_query_binding::UiProjectionInputSlot::for_certification(5),
    );
    let owner = UiSelectionOwnerIdentity::new(
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().expect("surface"),
        crate::graph::UiGraphNodeIdentity::new(72),
        family,
    );
    let keys = [11_u64, 12, 13]
        .into_iter()
        .map(|value| {
            UiSelectionStableKey::new(crate::runtime::UiApplicationItemKey::new(
                family,
                core::num::NonZeroU64::new(value).unwrap(),
            ))
        })
        .collect::<Vec<_>>();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(
            UiSelectionRegistration::new(
                owner,
                incarnation(),
                UiSelectionPolicy::Single,
                keys.clone(),
                UiSelectionCatalogPosture::Complete,
            )
            .unwrap()
            .with_catalog_revision(19),
        )
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[1]),
        )
        .unwrap();

    assert_eq!(state.suspend_projection_catalogs(), 1);
    assert_eq!(
        state.apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[0]),
        ),
        Err(UiSelectionRequestDenial::CatalogUnavailable)
    );
    assert!(state.family_requires_catalog_reconciliation(family, 19));
    assert_eq!(
        state
            .reconcile_projection_catalog(
                family,
                19,
                &[
                    core::num::NonZeroU64::new(11).unwrap(),
                    core::num::NonZeroU64::new(12).unwrap(),
                    core::num::NonZeroU64::new(13).unwrap(),
                ],
                UiSelectionCatalogPosture::Complete,
            )
            .unwrap(),
        1
    );
    assert_eq!(
        state
            .selected(owner)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![keys[1]]
    );
    assert!(state.catalog_is_current(owner, incarnation(), 19));
}

#[test]
fn suspension_reserves_one_revision_before_mutating_any_owner() {
    let family = crate::runtime::UiApplicationItemKeyFamily::from_projection_input(
        worth_ui_query_binding::UiProjectionInputSlot::for_certification(5),
    );
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    let owners = [71, 72].map(|node| {
        UiSelectionOwnerIdentity::new(
            surface,
            crate::graph::UiGraphNodeIdentity::new(node),
            family,
        )
    });
    for owner in owners {
        state
            .synchronize(
                UiSelectionRegistration::new(
                    owner,
                    incarnation(),
                    UiSelectionPolicy::Single,
                    vec![],
                    UiSelectionCatalogPosture::Complete,
                )
                .unwrap()
                .with_catalog_revision(19),
            )
            .unwrap();
    }
    state.revision = u64::MAX;
    let predecessor = state.appearance_owner_snapshot();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.suspend_projection_catalogs();
    }))
    .is_err());
    assert_eq!(state.appearance_owner_snapshot(), predecessor);
    for owner in owners {
        assert!(state.catalog_is_current(owner, incarnation(), 19));
    }

    state.revision = u64::MAX - 1;
    assert_eq!(state.suspend_projection_catalogs(), 2);
    assert_eq!(state.revision, u64::MAX);
    let suspended = state.appearance_owner_snapshot();
    assert_eq!(state.suspend_projection_catalogs(), 0);
    assert_eq!(state.appearance_owner_snapshot(), suspended);
    for owner in owners {
        assert!(!state.catalog_is_current(owner, incarnation(), 19));
    }
}
