use std::sync::Arc;

#[path = "test_support.rs"]
mod support;

#[test]
fn appearance_theme_value_truth_detects_legacy_equal_appearance_change() {
    let (session, mut state, surface, _role, _index) =
        support::owner_state_fixture(support::single_bundle([68, 85, 102, 255]), None);
    let token = support::theme_token();
    let change = support::theme_change(token.clone(), 0, support::legacy_color("#112233"));
    let before = Arc::clone(&state.token_values);
    let update = state
        .prepare_theme_values_for_appearance(
            std::slice::from_ref(&change),
            session.capabilities().appearance_themes().unwrap(),
            &session.active_generation_identity(),
        )
        .unwrap();

    assert_eq!(&*before, &*update.token_values);
    assert_eq!(update.changed_tokens(), &[token.clone()]);
    assert_eq!(update.theme_revision, 1);
    assert_eq!(update.mutable_token_revisions.get(&token), Some(&1));
    assert_eq!(
        update.appearance_theme_values[&surface]
            .values()
            .get(&token),
        Some(&support::typed_color([17, 34, 51, 255]))
    );
    state.commit_theme_values(update, None).unwrap();
    assert_eq!(state.theme_revision, 1);
    assert_eq!(state.mutable_token_revisions.get(&token), Some(&1));
    let _ = session.shutdown();
}

#[test]
fn appearance_theme_value_truth_noop_consumes_mutation_revision_only() {
    let (session, mut state, surface, _role, _index) =
        support::owner_state_fixture(support::single_bundle([17, 34, 51, 255]), None);
    let token = support::theme_token();
    let change = support::theme_change(token.clone(), 0, support::legacy_color("#112233"));
    let update = state
        .prepare_theme_values_for_appearance(
            std::slice::from_ref(&change),
            session.capabilities().appearance_themes().unwrap(),
            &session.active_generation_identity(),
        )
        .unwrap();

    assert!(update.changed_tokens().is_empty());
    assert!(update.semantic_presentation_revisions.is_empty());
    assert_eq!(update.theme_revision, 0);
    state.commit_theme_values(update, None).unwrap();
    assert_eq!(state.mutable_token_revisions.get(&token), Some(&1));
    assert_eq!(state.theme_revision, 0);
    assert!(state.pending_theme_tokens.is_empty());
    assert!(state.pending_appearance_invalidation.is_none());
    assert!(!state.theme_values_source().has_theme_changes());
    assert_eq!(
        state.appearance_theme_values[&surface].values().get(&token),
        Some(&support::typed_color([17, 34, 51, 255]))
    );
    let _ = session.shutdown();
}

#[test]
fn appearance_theme_value_truth_unions_aliases_once_when_one_binding_changes() {
    let (mut session, mut state, first_surface, role, _index) = support::owner_state_fixture(
        support::multi_bundle(),
        Some(support::global_alias_descriptor()),
    );
    let second_surface = session.create_semantic_surface().unwrap();
    let second =
        support::issue_capability(&session, &role, second_surface, "theme.truth.surface_b");
    state
        .materialize_initial_appearance_theme_binding(second)
        .unwrap();

    let token = support::theme_token();
    let alias = support::global_alias_token();
    let change = support::theme_change(token.clone(), 0, support::legacy_color("#112233"));
    let update = state
        .prepare_theme_values_for_appearance(
            std::slice::from_ref(&change),
            session.capabilities().appearance_themes().unwrap(),
            &session.active_generation_identity(),
        )
        .unwrap();
    let mut expected = vec![token, alias];
    expected.sort();
    assert_eq!(update.changed_tokens(), expected.as_slice());
    assert_eq!(update.appearance_theme_values.len(), 2);
    assert_eq!(update.theme_revision, 1);
    assert!(update.appearance_theme_values[&first_surface]
        .values()
        .contains_key(&support::theme_token()));
    let _ = session.shutdown();
}

#[test]
fn appearance_theme_value_truth_queue_failure_is_atomic_after_preparation() {
    let (session, mut state, surface, role, index) =
        support::owner_state_fixture(support::single_bundle([68, 85, 102, 255]), None);
    let token = support::theme_token();
    let change = support::theme_change(token.clone(), 0, support::legacy_color("#112233"));
    let batch = crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_slot(
        &index,
        token.as_str(),
        token.as_str(),
    )
    .unwrap();
    assert!(batch.selected_count() > 0);
    let before_values = Arc::clone(&state.token_values);
    let before_typed = state.appearance_theme_values.clone();
    let before_revisions = state.mutable_token_revisions.clone();
    let before_theme_revision = state.theme_revision;
    let before_pending_tokens = state.pending_theme_tokens.clone();
    let before_pending_batch = state.pending_appearance_invalidation.clone();
    let before_binding = state
        .active_appearance_theme_binding(surface)
        .unwrap()
        .clone();
    state.next_appearance_batch_revision = u64::MAX;

    let update = state
        .prepare_theme_values_for_appearance(
            std::slice::from_ref(&change),
            session.capabilities().appearance_themes().unwrap(),
            &session.active_generation_identity(),
        )
        .unwrap();
    assert!(state.commit_theme_values(update, Some(batch)).is_err());
    assert!(Arc::ptr_eq(&before_values, &state.token_values));
    assert_eq!(state.appearance_theme_values, before_typed);
    assert_eq!(state.mutable_token_revisions, before_revisions);
    assert_eq!(state.theme_revision, before_theme_revision);
    assert_eq!(state.pending_theme_tokens, before_pending_tokens);
    assert_eq!(state.pending_appearance_invalidation, before_pending_batch);
    assert_eq!(state.next_appearance_batch_revision, u64::MAX);
    assert_eq!(
        state.active_appearance_theme_binding(surface).unwrap(),
        &before_binding
    );
    assert_eq!(
        role.role(),
        before_binding.capability().required_roles()[0].identity()
    );
    let _ = session.shutdown();
}

#[test]
fn appearance_theme_value_truth_rejects_conflicting_canonical_alias_values_in_both_orders() {
    let first = support::theme_change(support::theme_token(), 0, support::legacy_color("#445566"));
    let second = support::theme_change(
        support::second_legacy_token(),
        0,
        support::legacy_color("#778899"),
    );
    for changes in [vec![first.clone(), second.clone()], vec![second, first]] {
        let (session, state, surface, role, _index) = support::owner_state_fixture(
            support::conflicting_bundle(),
            Some(support::second_legacy_descriptor()),
        );
        let themes = session.capabilities().appearance_themes().unwrap();
        let generation = session.active_generation_identity();
        assert_eq!(
            themes.catalog().resolved_target(&support::theme_token()),
            themes
                .catalog()
                .resolved_target(&support::second_legacy_token())
        );
        let before_values = Arc::clone(&state.token_values);
        let before_typed = state.appearance_theme_values.clone();
        let before_revisions = state.mutable_token_revisions.clone();
        let before_theme_revision = state.theme_revision;
        let before_pending_tokens = state.pending_theme_tokens.clone();
        let before_next_batch_revision = state.next_appearance_batch_revision;
        let before_binding = state
            .active_appearance_theme_binding(surface)
            .unwrap()
            .clone();

        assert!(state
            .prepare_theme_values_for_appearance(&changes, themes, &generation)
            .is_err());
        assert!(Arc::ptr_eq(&before_values, &state.token_values));
        assert_eq!(state.appearance_theme_values, before_typed);
        assert_eq!(state.mutable_token_revisions, before_revisions);
        assert_eq!(state.theme_revision, before_theme_revision);
        assert_eq!(state.pending_theme_tokens, before_pending_tokens);
        assert_eq!(
            state.next_appearance_batch_revision,
            before_next_batch_revision
        );
        assert_eq!(
            state.active_appearance_theme_binding(surface).unwrap(),
            &before_binding
        );
        assert_eq!(
            role.role(),
            before_binding.capability().required_roles()[0].identity()
        );
        let _ = session.shutdown();
    }
}
