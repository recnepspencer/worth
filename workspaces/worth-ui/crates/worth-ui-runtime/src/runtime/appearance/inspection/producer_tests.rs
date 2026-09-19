use worth_ui_dsl::{UiAppearanceAspect, UiAppearanceAxisClass, UiThemeColor, UiThemeValue};
use worth_ui_inspection::{
    UiAppearanceInspectionChangeDistinctions, UiAppearanceInspectionCost,
    UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
    UiAppearanceInspectionExplanation, UiAppearanceInspectionMountedMechanic,
    UiAppearanceInspectionOutcome, UiAppearanceInspectionPhysicalSuppression,
    UiAppearanceInspectionQuery, UiAppearanceInspectionSourceSpan, UiAppearanceInspectionSupport,
    UiAppearanceInspectionValue, UiAppearanceInspectionWorld, UiEvidenceAuthorityGeneration,
};

use super::producer::UiAppearanceInspectionProducer;

#[test]
fn inspection_store_bounds_entries_and_eviction_tombstones() {
    let mut producer = UiAppearanceInspectionProducer::new_for_test();
    for graph_node_digest in 0..130 {
        let query = query(graph_node_digest, UiAppearanceAspect::Background);
        producer.record(
            query,
            explanation(query, UiAppearanceInspectionSupport::Supported),
        );
    }

    let outcomes = (0..130)
        .map(|graph_node_digest| {
            producer.query(query(graph_node_digest, UiAppearanceAspect::Background))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, UiAppearanceInspectionOutcome::Found(_)))
            .count(),
        64
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, UiAppearanceInspectionOutcome::Expired))
            .count(),
        64
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, UiAppearanceInspectionOutcome::Unavailable))
            .count(),
        2
    );

    assert!(matches!(
        producer.query(query(64, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Expired
    ));
    assert!(matches!(
        producer.query(query(0, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Unavailable
    ));
    assert!(matches!(
        producer.query(query(129, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Found(_)
    ));

    let replacement = query(64, UiAppearanceAspect::Background);
    producer.record(
        replacement,
        explanation(replacement, UiAppearanceInspectionSupport::Supported),
    );
    assert!(matches!(
        producer.query(replacement),
        UiAppearanceInspectionOutcome::Found(_)
    ));
    assert!(matches!(
        producer.query(query(65, UiAppearanceAspect::Background)),
        UiAppearanceInspectionOutcome::Expired
    ));
}

#[test]
fn unsupported_inspection_is_not_presented_as_a_resolved_value() {
    let mut producer = UiAppearanceInspectionProducer::new_for_test();
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
    let mut producer = UiAppearanceInspectionProducer::new_for_test();
    let old = query(7, UiAppearanceAspect::Background);
    producer.record(
        old,
        explanation(old, UiAppearanceInspectionSupport::Supported),
    );
    producer.replace_test_world(UiAppearanceInspectionWorld::new(
        1,
        UiEvidenceAuthorityGeneration::new(3),
        4,
    ));

    assert_eq!(
        producer.query(old),
        UiAppearanceInspectionOutcome::WrongWorld
    );

    let current = query_in_world(
        UiAppearanceInspectionWorld::new(1, UiEvidenceAuthorityGeneration::new(3), 4),
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
        UiAppearanceInspectionWorld::new(1, UiEvidenceAuthorityGeneration::new(2), 3),
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
        worth_ui_inspection::UiAppearanceInspectionValueSource::ThemeSlot {
            selected: "surface.background".into(),
            terminal: "surface.background".into(),
        },
        support,
        UiAppearanceInspectionValue::Resolved(UiThemeValue::Color(UiThemeColor::from_channels([
            1, 2, 3, 255,
        ]))),
        UiAppearanceInspectionChangeDistinctions::new(false, false, false, false, false, false),
        UiAppearanceInspectionMountedMechanic::NotAttempted,
        UiAppearanceInspectionPhysicalSuppression::NotAttempted,
        query.graph_node_digest(),
        UiAppearanceInspectionEvidence::new(1, 1, [0; 6]),
        UiAppearanceInspectionCost::new(0, 1, 1, 1, 0),
    )
}
