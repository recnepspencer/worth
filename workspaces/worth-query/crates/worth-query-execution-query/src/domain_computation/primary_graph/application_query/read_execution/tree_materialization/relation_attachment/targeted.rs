use std::collections::BTreeSet;

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};

use super::super::relation_distribution::distribute_relation_rows;
use super::super::relation_target_filter::retain_matching_targets;
use super::super::{
    project_nodes, ActiveResultTreeCollectionSelection, ResultTreeWork, TargetedCollectionChild,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};
use super::{traversal_denial, work_limit_denial};

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_targeted_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: &crate::domain_computation::primary_graph::application_query::read_execution::tree_materialization::read_basis::ResultTreeRead<'_>,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    target: &TargetedCollectionChild,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let [parent] = parents else {
        return Err(traversal_denial(relation.result_path()));
    };
    let layout = graph
        .relation(relation.relation())
        .ok_or_else(|| traversal_denial(relation.result_path()))?;
    let frontier = BTreeSet::from([target.child_entity_id]);
    work.checkpoint(relation.result_path())?;
    let adjacency = match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => projection
            .bounded_incoming_relations_for_frontier(execution, &frontier, layout.kind, None)
            .map_err(|cause| {
                crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(
                    cause,
                    relation.result_path(),
                )
            })?,
        ApplicationQueryResultTraversalDirection::Reverse => projection
            .bounded_outgoing_relations_for_frontier(execution, &frontier, layout.kind, None)
            .map_err(|cause| {
                crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(
                    cause,
                    relation.result_path(),
                )
            })?,
    }
    .map_err(|_| work_limit_denial(relation.result_path()))?;
    work.charge_adjacency(
        adjacency.adjacency_lists_read(),
        adjacency.relation_records_examined(),
        adjacency.endpoint_records_reserved(),
        relation.result_path(),
    )?;
    let matching_memberships = adjacency
        .into_records()
        .into_iter()
        .filter(|record| match relation.direction() {
            ApplicationQueryResultTraversalDirection::Forward => {
                record.source == parent.entity_id() && record.target == target.child_entity_id
            }
            ApplicationQueryResultTraversalDirection::Reverse => {
                record.source == target.child_entity_id && record.target == parent.entity_id()
            }
        })
        .count();
    if matching_memberships != 1 {
        return Err(traversal_denial(relation.result_path()));
    }
    let mut filtered_target = vec![target.child_entity_id];
    retain_matching_targets(
        execution,
        projection,
        graph,
        governance,
        parameters,
        relation,
        &mut filtered_target,
        |entity| *entity,
        work,
    )?;
    if filtered_target.len() != 1 {
        return Err(traversal_denial(relation.result_path()));
    }
    let mut nested_selection = ActiveResultTreeCollectionSelection::Complete;
    let children = project_nodes(
        execution,
        read,
        projection,
        graph,
        contract,
        governance,
        parameters,
        Some(relation.result_path_identity()),
        relation.result_path(),
        relation.child_entity(),
        &[target.child_entity_id],
        work,
        &mut nested_selection,
    )?;
    distribute_relation_rows(
        parents,
        relation,
        vec![1],
        children,
        contract,
        governance,
        work,
        true,
        vec![usize::from(relation.predicate().is_some())],
        relation
            .predicate()
            .map(|_| vec![target.child_entity_id])
            .unwrap_or_default(),
    )
}
