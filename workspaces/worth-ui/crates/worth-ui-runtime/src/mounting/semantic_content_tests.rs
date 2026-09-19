use std::sync::Arc;

use super::{UiMountedSemanticContentInput, UiMountedSemanticTextValueDirective};

#[test]
fn application_presentation_merges_with_disjoint_rebind_content() {
    let query = crate::graph::UiGraphNodeIdentity::new(80_001);
    let application = crate::graph::UiGraphNodeIdentity::new(80_002);
    let mut content = scalar(query, "query", "PENDING");
    let application_content = scalar(application, "copy", "");

    content
        .merge_application_presentation(application_content)
        .expect("disjoint Query and application lanes merge");

    assert!(content.get(query).is_some());
    assert!(content.get(application).is_some());
}

#[test]
fn application_presentation_merge_rejects_overlap_without_partial_mutation() {
    let retained = crate::graph::UiGraphNodeIdentity::new(80_003);
    let overlap = crate::graph::UiGraphNodeIdentity::new(80_004);
    let new_row = crate::graph::UiGraphNodeIdentity::new(80_005);
    let mut content = scalar(retained, "retained", "CURRENT");
    content
        .insert_scalar(
            overlap,
            UiMountedSemanticTextValueDirective::Replace(Arc::from("query")),
            Arc::from("PENDING"),
        )
        .unwrap();
    let mut application = scalar(new_row, "copy", "");
    application
        .insert_scalar(
            overlap,
            UiMountedSemanticTextValueDirective::Replace(Arc::from("collision")),
            Arc::from(""),
        )
        .unwrap();

    assert_eq!(
        content.merge_application_presentation(application),
        Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution)
    );
    assert!(content.get(retained).is_some());
    assert!(content.get(overlap).is_some());
    assert!(content.get(new_row).is_none());
}

#[test]
fn unchanged_application_source_cannot_override_a_predecessor_or_query_owner() {
    let node = crate::graph::UiGraphNodeIdentity::new(80_006);
    let mut application = scalar(node, "published copy", "");
    application.retain_application_text_source(node);
    assert!(
        application.is_empty(),
        "a completion source is not a content change"
    );
    assert_eq!(application.graph_nodes().len(), 0);
    assert_eq!(application.text_for_lowering(node, true), (None, 0));
    let (Some(super::UiMountedSemanticTextContent::Scalar(restored)), 1) =
        application.text_for_lowering(node, false)
    else {
        panic!("missing occurrence obtains the captured scalar source");
    };
    assert_eq!(
        restored.value(),
        &UiMountedSemanticTextValueDirective::Replace(Arc::from("published copy"))
    );
    assert!(
        application
            .insert_scalar(
                node,
                UiMountedSemanticTextValueDirective::Preserve,
                Arc::from("")
            )
            .is_err(),
        "duplicate graph ownership must deny even when the first row is unchanged"
    );
    let mut query = scalar(node, "query copy", "");
    let previous = query.clone();
    assert_eq!(
        query.merge_application_presentation(application),
        Err(crate::mounting::UiMountedProjectionDenial::DuplicateLaneContribution)
    );
    assert_eq!(query, previous, "ownership denial is atomic");
}

fn scalar(
    node: crate::graph::UiGraphNodeIdentity,
    value: &'static str,
    posture: &'static str,
) -> UiMountedSemanticContentInput {
    let mut content = UiMountedSemanticContentInput::empty();
    content
        .insert_scalar(
            node,
            UiMountedSemanticTextValueDirective::Replace(Arc::from(value)),
            Arc::from(posture),
        )
        .unwrap();
    content
}
