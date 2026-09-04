use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::{UiApplicationPresentationState, UiApplicationSemanticTextRow};

#[test]
fn complete_projection_retains_current_paint_after_incremental_projection_commits() {
    let token =
        crate::capability::ThemeTokenId::new("theme.test.text").expect("test theme-token identity");
    let color = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#ffffff").expect("test theme-token value"),
    );
    let node = crate::graph::UiGraphNodeIdentity::new(91_001);
    let row = UiApplicationSemanticTextRow {
        graph_node: Some(node),
        value: Some(Arc::from("current text")),
        contract: crate::capability::ComponentSemanticTextContract::body_default(token.clone(), 1),
        semantic_revision: 1,
        presentation_revision: 2,
        projected_presentation_revision: None,
    };
    let mut state = UiApplicationPresentationState {
        rows: HashMap::from([(Box::<str>::from("component:test"), row)]),
        token_values: Arc::new(BTreeMap::from([(token.clone(), color.clone())])),
        resolved_targets: BTreeMap::from([(token.clone(), token.clone())]),
        mutable_token_revisions: BTreeMap::from([(token.clone(), 1)]),
        theme_revision: 0,
        pending_appearance_invalidation: None,
        pending_theme_tokens: std::collections::BTreeSet::new(),
        next_appearance_batch_revision: 1,
        appearance_theme_values: BTreeMap::new(),
        appearance_theme_state: Default::default(),
    };

    let incremental = state.project().expect("current row projects incrementally");
    state.commit(&incremental);
    assert!(state
        .project()
        .expect("committed projection")
        .content()
        .is_empty());

    let complete = state
        .project_complete()
        .expect("complete current projection");
    let complete_content = complete.content();
    let crate::mounting::UiMountedSemanticTextContent::Scalar(content) =
        complete_content.get(node).expect("current row is retained")
    else {
        panic!("current row remains scalar semantic text");
    };
    assert_eq!(
        content
            .formatting()
            .and_then(|formatting| formatting.token_value(&token)),
        Some(&color)
    );
    assert!(
        content.posture().trim().is_empty(),
        "application-authored copy has no synthetic user-visible posture"
    );
}

#[test]
fn theme_update_is_transactional_and_fans_out_to_alias_consumers() {
    let root = crate::capability::ThemeTokenId::new("theme.test.root").expect("root token");
    let alias = crate::capability::ThemeTokenId::new("theme.test.alias").expect("alias token");
    let initial = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#2f81f7").expect("initial color"),
    );
    let successor = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#3fb950").expect("successor color"),
    );
    let node = crate::graph::UiGraphNodeIdentity::new(91_002);
    let row = UiApplicationSemanticTextRow {
        graph_node: Some(node),
        value: Some(Arc::from("current text")),
        contract: crate::capability::ComponentSemanticTextContract::body_default(alias.clone(), 1),
        semantic_revision: 1,
        presentation_revision: 2,
        projected_presentation_revision: Some(2),
    };
    let mut state = UiApplicationPresentationState {
        rows: HashMap::from([(Box::<str>::from("component:test"), row)]),
        token_values: Arc::new(BTreeMap::from([
            (root.clone(), initial.clone()),
            (alias.clone(), initial),
        ])),
        resolved_targets: BTreeMap::from([
            (root.clone(), root.clone()),
            (alias.clone(), root.clone()),
        ]),
        mutable_token_revisions: BTreeMap::from([(root.clone(), 0)]),
        theme_revision: 0,
        pending_appearance_invalidation: None,
        pending_theme_tokens: std::collections::BTreeSet::new(),
        next_appearance_batch_revision: 1,
        appearance_theme_values: BTreeMap::new(),
        appearance_theme_state: Default::default(),
    };

    let change =
        crate::facade::entry::UiNativeThemeTokenValueChange::new(root.clone(), successor.clone())
            .expect("valid successor");
    let update = state
        .prepare_theme_values(std::slice::from_ref(&change))
        .expect("current revision prepares");
    assert_eq!(update.changed_tokens(), &[alias.clone(), root.clone()]);
    state
        .commit_theme_values(update, None)
        .expect("prepared transaction commits");

    assert_eq!(state.token_values.get(&root), Some(&successor));
    assert_eq!(state.token_values.get(&alias), Some(&successor));
    assert_eq!(state.mutable_token_revisions.get(&root), Some(&1));
    assert_eq!(state.theme_revision, 1);
    assert_eq!(state.rows["component:test"].presentation_revision, 3);
    assert!(!state.theme_values_source().has_theme_changes());
    assert!(state.pending_theme_tokens.is_empty());

    let before_values = Arc::clone(&state.token_values);
    let before_revisions = state.mutable_token_revisions.clone();
    assert!(state.prepare_theme_values(&[change]).is_err());
    assert!(Arc::ptr_eq(&before_values, &state.token_values));
    assert_eq!(state.mutable_token_revisions, before_revisions);
}

#[test]
fn theme_update_queue_overflow_is_atomic_before_value_commit() {
    let token = crate::capability::ThemeTokenId::new(
        crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_TOKEN,
    )
    .expect("theme token");
    let initial = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#112233").expect("initial color"),
    );
    let successor = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex("#445566").expect("successor color"),
    );
    let row = UiApplicationSemanticTextRow {
        graph_node: None,
        value: Some(Arc::from("current text")),
        contract: crate::capability::ComponentSemanticTextContract::body_default(token.clone(), 1),
        semantic_revision: 1,
        presentation_revision: 2,
        projected_presentation_revision: Some(2),
    };
    let mut state = UiApplicationPresentationState {
        rows: HashMap::from([(Box::<str>::from("component:test"), row)]),
        token_values: Arc::new(BTreeMap::from([(token.clone(), initial)])),
        resolved_targets: BTreeMap::from([(token.clone(), token.clone())]),
        mutable_token_revisions: BTreeMap::from([(token.clone(), 0)]),
        theme_revision: 0,
        pending_appearance_invalidation: None,
        pending_theme_tokens: std::collections::BTreeSet::new(),
        next_appearance_batch_revision: u64::MAX,
        appearance_theme_values: BTreeMap::new(),
        appearance_theme_state: Default::default(),
    };
    let role = crate::runtime::tests::appearance_component_session_test_support::
        validation_background_role(
            crate::runtime::tests::appearance_component_session_test_support::APPEARANCE_TOKEN,
        );
    let application = crate::runtime::tests::appearance_component_session_test_support::
        appearance_component_builder(&role)
        .with_rust_authored_declaration_fixture(
            crate::runtime::tests::appearance_component_session_test_support::appearance_fixture(
                &role,
            ),
        )
        .freeze()
        .map(crate::runtime::tests::appearance_theme_test_support::activate)
        .expect("the canonical appearance source application must prepare");
    let invalidation = crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_slot(
        application.prepared_authority().consumed_fact_index(),
        token.as_str(),
        token.as_str(),
    )
    .expect("the canonical appearance slot batch must be available");
    assert_ne!(invalidation.selected_count(), 0);
    let change = crate::facade::entry::UiNativeThemeTokenValueChange::new(token, successor)
        .expect("valid successor");
    let update = state
        .prepare_theme_values(std::slice::from_ref(&change))
        .expect("current revision prepares");
    let before_values = Arc::clone(&state.token_values);
    let before_revisions = state.mutable_token_revisions.clone();
    let before_rows = state.rows["component:test"].presentation_revision;
    let before_theme_revision = state.theme_revision;
    let before_pending_tokens = state.pending_theme_tokens.clone();
    let before_pending_batch = state.pending_appearance_invalidation.clone();
    let before_next_batch_revision = state.next_appearance_batch_revision;

    assert!(state
        .commit_theme_values(update, Some(invalidation))
        .is_err());
    assert!(Arc::ptr_eq(&before_values, &state.token_values));
    assert_eq!(state.mutable_token_revisions, before_revisions);
    assert_eq!(
        state.rows["component:test"].presentation_revision,
        before_rows
    );
    assert_eq!(state.theme_revision, before_theme_revision);
    assert_eq!(state.pending_theme_tokens, before_pending_tokens);
    assert_eq!(state.pending_appearance_invalidation, before_pending_batch);
    assert_eq!(
        state.next_appearance_batch_revision,
        before_next_batch_revision
    );
    assert!(state.appearance_theme_values.is_empty());
}
