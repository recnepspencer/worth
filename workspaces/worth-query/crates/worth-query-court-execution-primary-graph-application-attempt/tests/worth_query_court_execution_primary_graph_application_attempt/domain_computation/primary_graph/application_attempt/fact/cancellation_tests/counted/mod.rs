//! Native entity, revision and scalar reads under the same actual fact door.
use crate::domain_computation::primary_graph::application_attempt::fact::cancellation_tests::*;
#[test]
fn fact_comparison_preserves_stops_at_every_consult() {
    with_indexed_fact(|runtime, execution, snapshot, original, _, relation_kind| {
        let WorthQueryApplicationObservedFact::IndexedEntitySelection {
            candidates,
            entity_kind,
            locator,
            value,
            ..
        } = original
        else {
            unreachable!()
        };
        let entity = candidates[0];
        let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
        let revision = view
            .entity_field_revision(execution, entity, locator)
            .unwrap();
        let direction = worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming;
        let adjacency_revision = view
            .bounded_adjacency_structural_revision(execution, entity, relation_kind, direction, 32)
            .unwrap()
            .revision();
        let endpoints = view
            .bounded_incoming_relations_of_kind(execution, entity, relation_kind, Some(32))
            .unwrap()
            .unwrap()
            .into_records()
            .into_iter()
            .map(|row| row.source)
            .collect::<Vec<_>>();
        let facts=[WorthQueryApplicationObservedFact::SourceEntity{entity_id:entity},WorthQueryApplicationObservedFact::SourceFieldRevision{entity_id:entity,locator:locator.clone(),native_revision:revision},WorthQueryApplicationObservedFact::Field{entity_id:entity,kind:*entity_kind,locator:locator.clone(),value:value.clone()},WorthQueryApplicationObservedFact::AbsentField{entity_id:entity,kind:*entity_kind,locator:locator.clone()},WorthQueryApplicationObservedFact::SourceAdjacencyRevision{relation_kind,anchor:entity,direction,native_revision:adjacency_revision,comparison_work_limit:32,endpoints:worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&endpoints,worth_execution::ExecutionAllocationPolicy::SystemAllocation,None).unwrap()}];
        for fact in &facts {
            worth_query_execution::test_read_stops::every_consult(|request|fact.remains_equal_in(runtime,request,snapshot).map_err(|denied|denied.kind()),Kind::Cancelled,Kind::ExecutionRequest(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource{denial:worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,partition_identity:None,policy_ancestor:None}));
        }
    });
}
