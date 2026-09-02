use worth_ui_dsl::{UiAppearanceAspect, UiAppearanceAxisClass, UiThemeColor, UiThemeValue};
use worth_ui_inspection::{
    UiAppearanceInspectionCost, UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
    UiAppearanceInspectionExplanation, UiAppearanceInspectionInvalidationCause,
    UiAppearanceInspectionMountedMechanic, UiAppearanceInspectionOutcome,
    UiAppearanceInspectionPhysicalSuppression, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSourceSpan, UiAppearanceInspectionSupport, UiAppearanceInspectionValue,
    UiAppearanceInspectionWorld,
};

use super::producer::UiAppearanceInspectionProducer;

#[test]
fn inspection_store_is_bounded_and_reports_eviction() {
    let mut producer = UiAppearanceInspectionProducer::new();
    for graph_node_digest in 0..65 {
        let query = query(graph_node_digest, UiAppearanceAspect::Background);
        producer.record(
            query,
            explanation(query, UiAppearanceInspectionSupport::Supported),
        );
    }

    assert!(matches!(
        producer.query(query(0, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Expired
    ));
    assert!(matches!(
        producer.query(query(64, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Found(_)
    ));

    let replacement = query(0, UiAppearanceAspect::Background);
    producer.record(
        replacement,
        explanation(replacement, UiAppearanceInspectionSupport::Supported),
    );
    assert!(matches!(
        producer.query(replacement),
        UiAppearanceInspectionOutcome::Found(_)
    ));
    assert!(matches!(
        producer.query(query(1, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Expired
    ));
}

#[test]
fn unsupported_inspection_is_not_presented_as_a_resolved_value() {
    let mut producer = UiAppearanceInspectionProducer::new();
    let query = query(7, UiAppearanceAspect::Outline);
    producer.record(
        query,
        explanation(query, UiAppearanceInspectionSupport::Unsupported),
    );

    assert_eq!(
        producer.query(query),
        UiAppearanceInspectionOutcome::Unsupported
    );
}

#[test]
fn retired_world_is_not_reinterpreted_as_expired_current_evidence() {
    let mut producer = UiAppearanceInspectionProducer::new();
    let old = query(7, UiAppearanceAspect::Background);
    producer.record(
        old,
        explanation(old, UiAppearanceInspectionSupport::Supported),
    );
    producer.reset_for_new_generation();

    assert_eq!(
        producer.query(old),
        UiAppearanceInspectionOutcome::WrongWorld
    );

    let current = query_in_world(
        UiAppearanceInspectionWorld::new(1, 3, 4),
        7,
        UiAppearanceAspect::Background,
    );
    producer.record(
        current,
        explanation(current, UiAppearanceInspectionSupport::Supported),
    );
    assert!(matches!(
        producer.query(current),
        UiAppearanceInspectionOutcome::Found(_)
    ));
}

fn query(graph_node_digest: u64, aspect: UiAppearanceAspect) -> UiAppearanceInspectionQuery {
    query_in_world(
        UiAppearanceInspectionWorld::new(1, 2, 3),
        graph_node_digest,
        aspect,
    )
}

fn query_in_world(
    world: UiAppearanceInspectionWorld,
    graph_node_digest: u64,
    aspect: UiAppearanceAspect,
) -> UiAppearanceInspectionQuery {
    UiAppearanceInspectionQuery::new(world, graph_node_digest, aspect)
}

fn explanation(
    query: UiAppearanceInspectionQuery,
    support: UiAppearanceInspectionSupport,
) -> UiAppearanceInspectionExplanation {
    UiAppearanceInspectionExplanation::new(
        query,
        "test.role",
        1,
        "test.theme",
        1,
        Box::<[UiAppearanceAxisClass]>::from([]),
        UiAppearanceInspectionDecisionCell::new(1, Box::<[UiAppearanceAxisClass]>::from([])),
        UiAppearanceInspectionSourceSpan::Unavailable,
        "surface.background",
        "surface.background",
        support,
        UiAppearanceInspectionValue::Resolved(UiThemeValue::Color(UiThemeColor::from_channels([
            1, 2, 3, 255,
        ]))),
        UiAppearanceInspectionInvalidationCause::NotAttributed,
        UiAppearanceInspectionMountedMechanic::NotEvaluated,
        UiAppearanceInspectionPhysicalSuppression::NotEvaluated,
        query.graph_node_digest(),
        UiAppearanceInspectionEvidence::new(1, 1, [0; 6]),
        UiAppearanceInspectionCost::new(0, 1, 1, 1, 0),
    )
}
