use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::{UiApplicationPresentationState, UiApplicationSemanticTextRow};

#[test]
fn complete_projection_retains_current_paint_after_incremental_projection_commits() {
    let token =
        crate::capability::ThemeTokenId::new("theme.test.text").expect("test theme-token identity");
    let color = crate::capability::ThemeTokenValue::color(
        crate::capability::UiThemeColor::parse("#ffffff").expect("test theme-token value"),
    );
    let node = crate::graph::UiGraphNodeIdentity::new(91_001);
    let row = UiApplicationSemanticTextRow {
        graph_node: Some(node),
        value: Some(Arc::from("current text")),
        contract: crate::capability::ComponentSemanticTextContract::body_default(token.clone(), 1),
        semantic_revision: 1,
        presentation_revision: 2,
        projected_presentation_revision: None,
        pending_publication_coverage: None,
    };
    let mut state = UiApplicationPresentationState {
        rows: HashMap::from([(Box::<str>::from("component:test"), row)]),
        token_values: Arc::new(BTreeMap::from([(token.clone(), color.clone())])),
        pending_appearance_invalidation: None,
        next_appearance_batch_revision: 1,
        appearance_theme_state: Default::default(),
    };

    let incremental = state.project().expect("current row projects incrementally");
    state.settle_published_text(&scoped(incremental.text_publication(), true));
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

    let older_publication = scoped(complete.text_publication(), true);
    state
        .admit_semantic_text(&[
            crate::facade::entry::UiNativeComponentSemanticTextChange::successor(
                "component:test",
                1,
                "newer text",
            )
            .unwrap(),
        ])
        .unwrap();
    state.settle_published_text(&older_publication);
    assert_eq!(state.rows["component:test"].presentation_revision, 3);
    assert_eq!(
        state.rows["component:test"].projected_presentation_revision,
        Some(2)
    );
    assert!(!state.project().unwrap().content().is_empty());
    let incomplete_scope = scoped(state.project().unwrap().text_publication(), false);
    state.settle_published_text(&incomplete_scope);
    assert!(!state.project().unwrap().content().is_empty());
    let current_publication = scoped(state.project().unwrap().text_publication(), true);
    state.settle_published_text(&current_publication);
    assert_eq!(
        state.rows["component:test"].projected_presentation_revision,
        Some(3)
    );
    assert!(state.project().unwrap().content().is_empty());
}

fn scoped(
    selection: super::UiApplicationTextRevisionSelection,
    selected: bool,
) -> super::UiApplicationTextPublication {
    let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    super::UiApplicationTextPublication {
        revisions: selection
            .revisions
            .into_vec()
            .into_iter()
            .map(|(identity, graph, revision)| {
                (
                    identity,
                    graph,
                    revision,
                    super::UiApplicationTextMountedCoverage::from_occurrences([(
                        instance,
                        incarnation,
                        selected,
                    )]),
                )
            })
            .collect(),
    }
}
