//! One configured kind applies to its bounded and complete compiled profiles.
use super::*;
#[test]
fn graph_support_configuration_cannot_leave_complete_profile_available() {
    for kind in [
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
        WorthQueryGraphReadAccessRequirementKind::VisitedSet,
    ] {
        let default = WorthQueryRuntimeSupportProfile::new([]);
        let rows = default.graph_index_inventory();
        assert_eq!(
            rows.rows()
                .iter()
                .filter(|row| row.requirement_kind() == &kind)
                .count(),
            2
        );
        let omitted = default
            .clone()
            .with_graph_index_support_omitted(kind.clone())
            .graph_index_inventory();
        assert!(omitted
            .rows()
            .iter()
            .all(|row| row.requirement_kind() != &kind));
        let unavailable = default
            .clone()
            .with_graph_index_temporarily_unavailable(kind.clone())
            .graph_index_inventory();
        let changed: Vec<_> = unavailable
            .rows()
            .iter()
            .filter(|row| row.requirement_kind() == &kind)
            .collect();
        assert_eq!(changed.len(), 2);
        assert!(changed
            .iter()
            .all(|row| row.posture() == &WorthQueryGraphIndexPosture::TemporarilyUnavailable));
        let directed = default
            .with_graph_index_supported_relation_direction(
                kind.clone(),
                WorthQueryAdmittedGraphReadRelationDirection::Descendant,
            )
            .graph_index_inventory();
        assert!(directed
            .rows()
            .iter()
            .filter(|row| row.requirement_kind() == &kind)
            .all(|row| row.supported_relation_direction()
                == Some(&WorthQueryAdmittedGraphReadRelationDirection::Descendant)));
    }
}
