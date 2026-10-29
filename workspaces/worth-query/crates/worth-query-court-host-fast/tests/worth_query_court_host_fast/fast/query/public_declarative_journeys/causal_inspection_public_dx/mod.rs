pub(crate) use worth_query_worlds::facade::query::public_declarative_journeys::causal_inspection_public_dx::*;

#[test]
fn causal_inspection_public_dx_signatures_are_referenced() {
    let _ = common_path_compiles
        as fn(
            QueryObservationReceipt,
            ScopedInspectionBasis,
            &RuntimeBridge,
        ) -> Result<
            QueryCausalInspectionArtifact,
            worth_query::facade::runtime::CausalInspectionMaterializationError,
        >;
    let _ = plan_inspection_compiles as fn(QueryObservationReceipt, ScopedInspectionBasis);
    let _ = support_discovery_compiles as fn();
    let _ = artifact_exploration_compiles as fn(QueryCausalInspectionArtifact);
    let _ = advanced_vocabulary_remains_available as fn();
}
