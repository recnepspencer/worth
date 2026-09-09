use super::state_test_fixture::*;
use super::*;

#[test]
fn single_multiple_and_range_actions_emit_compact_stable_key_deltas() {
    let owner = owner();
    let keys = (1..=5).map(key).collect::<Vec<_>>();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::MultipleWithRange,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();

    let first = state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[1]),
        )
        .unwrap();
    assert_eq!(first.added(), &[keys[1]]);
    assert_eq!(first.selected_count(), 1);

    let range = state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectRange {
                target: keys[4],
                extend: false,
            },
        )
        .unwrap();
    assert_eq!(range.added(), &keys[2..=4]);
    assert!(range.removed().is_empty());
    assert_eq!(range.selected_count(), 4);
    assert_eq!(range.candidates_visited(), 5);

    let toggle = state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::ToggleMultiple(keys[3]),
        )
        .unwrap();
    assert_eq!(toggle.removed(), &[keys[3]]);
    assert_eq!(toggle.selected_count(), 3);
}

#[test]
fn range_default_and_disabled_key_preservation_change_owner_behavior() {
    let owner = owner();
    let keys = (41..=43).map(key).collect::<Vec<_>>();
    let policy = crate::declaration::UiSelectionPolicy::range().with_stable_key_preservation(false);
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate_with_policy(policy);
    assert_eq!(
        state.default_owner_policy(),
        UiSelectionPolicy::MultipleWithRange
    );
    state
        .synchronize(registration(
            owner,
            state.default_owner_policy(),
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[0]),
        )
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectRange {
                target: keys[2],
                extend: false,
            },
        )
        .expect("range default admits range selection");

    let reconciliation = state
        .synchronize(registration(
            owner,
            state.default_owner_policy(),
            keys[1..].to_vec(),
            UiSelectionCatalogPosture::Partial,
        ))
        .unwrap();
    assert_eq!(reconciliation.delta().removed(), &[keys[0]]);
    assert_eq!(
        reconciliation.missing_keys_preserved_for_partial_catalog(),
        0
    );
}

#[test]
fn extending_a_range_preserves_the_predecessor_selection() {
    let owner = owner();
    let keys = (20..=25).map(key).collect::<Vec<_>>();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::MultipleWithRange,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[1]),
        )
        .unwrap();
    state
        .apply(owner, incarnation(), UiSelectionRequest::Add(keys[5]))
        .unwrap();

    let delta = state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectRange {
                target: keys[3],
                extend: true,
            },
        )
        .unwrap();
    assert_eq!(delta.added(), &keys[2..=3]);
    assert!(delta.removed().is_empty());
    assert_eq!(
        state
            .selected(owner)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![keys[1], keys[2], keys[3], keys[5]]
    );
}

#[test]
fn catalog_rejects_a_key_from_another_application_family() {
    let owner = owner();
    let foreign = UiSelectionStableKey::new(crate::runtime::UiApplicationItemKey::new(
        crate::runtime::UiApplicationItemKeyFamily::new(core::num::NonZeroU64::new(99).unwrap()),
        core::num::NonZeroU64::new(1).unwrap(),
    ));
    assert_eq!(
        UiSelectionRegistration::new(
            owner,
            incarnation(),
            UiSelectionPolicy::Single,
            vec![foreign],
            UiSelectionCatalogPosture::Complete,
        ),
        Err(UiSelectionRequestDenial::ForeignItemKeyFamily)
    );
}

#[test]
fn reorder_preserves_selection_and_complete_removal_reconciles_by_key() {
    let owner = owner();
    let keys = (10..=13).map(key).collect::<Vec<_>>();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::ToggleMultiple(keys[1]),
        )
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::ToggleMultiple(keys[3]),
        )
        .unwrap();

    let reordered = state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            keys.iter().rev().copied().collect(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert!(reordered.order_changed());
    assert!(reordered.delta().removed().is_empty());
    assert_eq!(reordered.delta().selected_count(), 2);

    let removed = state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            vec![keys[0], keys[1], keys[2]],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert_eq!(removed.delta().removed(), &[keys[3]]);
    assert_eq!(removed.delta().selected_count(), 1);
}

#[test]
fn partial_catalog_never_claims_an_absent_key_was_removed() {
    let owner = owner();
    let keys = vec![key(21), key(22)];
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[1]),
        )
        .unwrap();
    let partial = state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            vec![keys[0]],
            UiSelectionCatalogPosture::Partial,
        ))
        .unwrap();
    assert!(partial.delta().removed().is_empty());
    assert_eq!(partial.missing_keys_preserved_for_partial_catalog(), 1);
    assert_eq!(state.selected(owner).unwrap().len(), 1);
}

#[test]
fn range_requires_owner_held_anchor_and_declared_range_policy() {
    let owner = owner();
    let keys = vec![key(31), key(32)];
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::MultipleWithRange,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert_eq!(
        state.apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectRange {
                target: keys[1],
                extend: false,
            },
        ),
        Err(UiSelectionRequestDenial::MissingRangeAnchor)
    );
}

#[test]
fn add_and_remove_obey_declared_multiple_policy() {
    let owner = owner();
    let keys = vec![key(41), key(42), key(43)];
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert_eq!(
        state
            .apply(owner, incarnation(), UiSelectionRequest::Add(keys[1]))
            .unwrap()
            .added(),
        &[keys[1]]
    );
    assert_eq!(
        state
            .apply(owner, incarnation(), UiSelectionRequest::Remove(keys[1]))
            .unwrap()
            .removed(),
        &[keys[1]]
    );
}

#[test]
fn thousand_key_catalog_keeps_single_key_mutation_work_constant() {
    let owner = owner();
    let keys = (1_000..2_024).map(key).collect::<Vec<_>>();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Multiple,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    let before = state.selection_keys_visited();
    let delta = state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::ToggleMultiple(keys[777]),
        )
        .unwrap();
    assert_eq!(delta.candidates_visited(), 1);
    assert_eq!(state.selection_keys_visited() - before, 1);
    assert_eq!(delta.added(), &[keys[777]]);
}

#[test]
fn shutdown_releases_every_selection_owner() {
    let owner = owner();
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            vec![key(51)],
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    assert_eq!(state.shutdown(), 1);
    assert_eq!(state.shutdown(), 0);
}

#[test]
fn failed_combined_rebind_and_action_preserves_predecessor_selection() {
    let owner = owner();
    let keys = vec![key(61), key(62)];
    let mut state = UiSelectionRuntimeState::new_session_restore_candidate();
    state
        .synchronize(registration(
            owner,
            UiSelectionPolicy::Single,
            keys.clone(),
            UiSelectionCatalogPosture::Complete,
        ))
        .unwrap();
    state
        .apply(
            owner,
            incarnation(),
            UiSelectionRequest::SelectSingle(keys[1]),
        )
        .unwrap();
    let replacement_incarnation = UiSelectionOwnerIncarnation::new(8).unwrap();
    let replacement = UiSelectionRegistration::new(
        owner,
        replacement_incarnation,
        UiSelectionPolicy::Single,
        keys,
        UiSelectionCatalogPosture::Complete,
    )
    .unwrap();
    assert_eq!(
        state.synchronize_and_apply(replacement, UiSelectionRequest::Add(key(61))),
        Err(UiSelectionRequestDenial::MultipleNotSupported)
    );
    assert!(state.selected(owner).unwrap().contains(&key(62)));
    assert_eq!(state.selected(owner).unwrap().len(), 1);
}
