use super::super::state_test_fixture::{incarnation, key, owner, registration};
use super::super::*;

#[test]
fn selection_cursor_only_changes_survive_empty_membership_delta_and_staging_comparison() {
    let owner = owner();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            vec![key(1), key(2), key(3)],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    for target in [key(1), key(2), key(3)] {
        state
            .apply(owner, incarnation(), UiSelectionRequest::Add(target))
            .unwrap();
    }
    let before = state.appearance_owner_snapshot();
    let delta = state
        .apply(owner, incarnation(), UiSelectionRequest::Add(key(2)))
        .unwrap();
    assert!(delta.added().is_empty());
    assert!(delta.removed().is_empty());
    assert_eq!(delta.positions().previous().anchor, Some(key(1)));
    assert_eq!(delta.positions().previous().cursor, Some(key(3)));
    assert_eq!(delta.positions().current().cursor, Some(key(2)));
    let after = state.appearance_owner_snapshot();
    assert_eq!(
        after.changes_since(&before),
        vec![UiSelectionAppearanceChange::Keys {
            owner,
            incarnation: incarnation(),
            keys: vec![key(2), key(3)].into(),
        }]
    );
    assert_eq!(
        before
            .posture_for(owner, key(3), incarnation())
            .unwrap()
            .source_bits(),
        (true, false, true)
    );
    assert_eq!(
        after
            .posture_for(owner, key(3), incarnation())
            .unwrap()
            .source_bits(),
        (true, false, false)
    );
    let noop = state
        .apply(owner, incarnation(), UiSelectionRequest::Add(key(2)))
        .unwrap();
    assert!(state
        .appearance_owner_snapshot()
        .changes_since(&after)
        .is_empty());
    assert!(
        !noop.has_same_effect_as(&delta),
        "same membership is insufficient when cursor effects differ"
    );
}

#[test]
fn selection_catalog_reconciliation_and_incarnation_retirement_keep_exact_scope() {
    let owner = owner();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            vec![key(1), key(2)],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(key(1)),
        )
        .unwrap();
    let before = state.appearance_owner_snapshot();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            vec![key(2)],
            UiSelectionCatalogPosture::Partial,
        ))
        .unwrap();
    let partial = state.appearance_owner_snapshot();
    assert!(partial.changes_since(&before).is_empty());
    let receipt = state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            vec![key(2)],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert_eq!(receipt.delta().removed(), &[key(1)]);
    assert_eq!(receipt.delta().positions().previous().anchor, Some(key(1)));
    assert_eq!(
        receipt.delta().positions().current(),
        UiSelectionPositions::default()
    );
    let complete = state.appearance_owner_snapshot();
    assert_eq!(
        complete.changes_since(&partial),
        vec![UiSelectionAppearanceChange::Keys {
            owner,
            incarnation: incarnation(),
            keys: vec![key(1)].into(),
        }]
    );
    let successor = UiSelectionOwnerIncarnation::new(8).unwrap();
    state
        .synchronize(
            UiSelectionRegistration::new(
                owner,
                successor,
                UiSelectionPolicy::Multiple,
                vec![key(2)],
                UiSelectionCatalogPosture::Complete,
            )
            .unwrap(),
        )
        .unwrap();
    let reincarnated = state.appearance_owner_snapshot();
    assert_eq!(
        reincarnated.changes_since(&complete),
        vec![UiSelectionAppearanceChange::Owner {
            owner,
            previous: Some(incarnation()),
            current: Some(successor),
        }]
    );
    assert_eq!(
        reincarnated.posture_for(owner, key(2), incarnation()),
        Err(UiSelectionAppearancePostureDenial::StaleOwnerIncarnation)
    );
    assert_eq!(
        state.retire_mounted_owner(owner.semantic_surface(), owner.graph_node(), successor),
        1
    );
    let retired = state.appearance_owner_snapshot();
    assert_eq!(
        retired.changes_since(&reincarnated),
        vec![UiSelectionAppearanceChange::Owner {
            owner,
            previous: Some(successor),
            current: None,
        }]
    );
    assert_eq!(
        before
            .posture_for(owner, key(1), incarnation())
            .unwrap()
            .source_bits(),
        (true, true, true)
    );
}

#[test]
fn selection_family_ambiguity_invalidates_the_surviving_mounted_owner() {
    let first = owner();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            first,
            UiSelectionPolicy::Single,
            vec![key(1)],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    let before = state.appearance_owner_snapshot();
    let family =
        crate::runtime::UiApplicationItemKeyFamily::new(core::num::NonZeroU64::new(9).unwrap());
    let second =
        UiSelectionOwnerIdentity::new(first.semantic_surface(), first.graph_node(), family);
    let second_key = UiSelectionStableKey::new(crate::runtime::UiApplicationItemKey::new(
        family,
        core::num::NonZeroU64::new(1).unwrap(),
    ));
    state
        .synchronize(registration(
            second,
            UiSelectionPolicy::Single,
            vec![second_key],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    let ambiguous = state.appearance_owner_snapshot();
    assert_eq!(
        ambiguous.posture_for(first, key(1), incarnation()),
        Err(UiSelectionAppearancePostureDenial::AmbiguousMountedOwner)
    );
    assert!(ambiguous
        .changes_since(&before)
        .contains(&UiSelectionAppearanceChange::Owner {
            owner: first,
            previous: Some(incarnation()),
            current: Some(incarnation()),
        }));
    state.retire_family(family);
    let recovered = state.appearance_owner_snapshot();
    assert!(recovered.posture_for(first, key(1), incarnation()).is_ok());
    assert!(recovered
        .changes_since(&ambiguous)
        .contains(&UiSelectionAppearanceChange::Owner {
            owner: first,
            previous: Some(incarnation()),
            current: Some(incarnation()),
        }));
    assert!(before.posture_for(first, key(1), incarnation()).is_ok());
}
