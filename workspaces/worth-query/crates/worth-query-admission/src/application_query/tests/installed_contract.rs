use super::{
    account_parameter, admitted_requirements, installed_query, ApplicationQueryParameterSet,
    WorthQueryApplicationQueryLane, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadResultPressure,
};
use crate::facade::application_query::admit_application_query_parameters;

#[test]
fn installed_application_graph_uses_canonical_requirement_derivation() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let requirements = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );

    for kind in [
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
        WorthQueryGraphReadAccessRequirementKind::VisitedSet,
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
        WorthQueryGraphReadAccessRequirementKind::ProofSupport,
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer,
        WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle,
    ] {
        assert!(requirements.contains_kind(&kind), "missing {kind:?}");
    }
    assert!(!requirements
        .contains_kind(&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport));
    assert!(requirements
        .rows()
        .iter()
        .all(|row| row.maximum_cardinality() == Some(32)));
    let ordering = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::OrderingSupport)
        .unwrap();
    assert_eq!(
        ordering.ordering_posture(),
        Some(&WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection)
    );
    assert_eq!(
        ordering.ordering_field_authorities()[0].collection_path(),
        "root/relation[0]"
    );
    let result_buffer = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
        .unwrap();
    assert_eq!(
        requirements
            .rows()
            .iter()
            .filter(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
            .count(),
        1
    );
    assert_eq!(
        result_buffer.result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::CollectionWide)
    );
}
