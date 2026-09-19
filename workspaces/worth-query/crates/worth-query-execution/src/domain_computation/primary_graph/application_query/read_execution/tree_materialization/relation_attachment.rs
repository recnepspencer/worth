use std::collections::{BTreeMap, BTreeSet};

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryResultTraversalDirection,
};
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::identity::EntityId;

use super::relation_distribution::distribute_relation_rows;
use super::relation_target_filter::retain_matching_targets;
use super::{
    project_nodes, ActiveResultTreeCollectionSelection, ResultTreeWork, TargetedCollectionChild,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::{
    read_execution::{read_execution_denial, WorthQueryApplicationReadExecutionDenialKind},
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};

mod omitted_relation;
mod ordered;
pub(super) use omitted_relation::advance_omitted_relation;
use ordered::attach_ordered_relation;

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    collection_selection: &mut ActiveResultTreeCollectionSelection,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let layout = graph
        .relation(relation.relation())
        .filter(|layout| {
            graph.entity_kind(relation.from()) == Some(layout.from)
                && graph.entity_kind(relation.to()) == Some(layout.to)
        })
        .ok_or_else(|| traversal_denial(relation.result_path()))?;
    let frontier = parents
        .iter()
        .map(WorthQueryApplicationProjectionNode::entity_id)
        .collect::<BTreeSet<_>>();
    match collection_selection {
        ActiveResultTreeCollectionSelection::Ordered(window)
            if window
                .request
                .as_ref()
                .is_some_and(|request| request.collection_path == relation.result_path()) =>
        {
            return attach_ordered_relation(
                runtime,
                projection,
                graph,
                contract,
                governance,
                parameters,
                relation,
                parents,
                work,
                window,
                result_buffer,
            );
        }
        ActiveResultTreeCollectionSelection::Targeted(target)
            if target.collection_path == relation.result_path() =>
        {
            return attach_targeted_relation(
                runtime,
                projection,
                graph,
                contract,
                governance,
                parameters,
                relation,
                parents,
                work,
                target,
                result_buffer,
            );
        }
        ActiveResultTreeCollectionSelection::Complete
        | ActiveResultTreeCollectionSelection::Ordered(_)
        | ActiveResultTreeCollectionSelection::Targeted(_) => {}
    }
    let read = match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => projection
            .bounded_outgoing_relations_for_frontier(&frontier, layout.kind, work.remaining_work()),
        ApplicationQueryResultTraversalDirection::Reverse => projection
            .bounded_incoming_relations_for_frontier(&frontier, layout.kind, work.remaining_work()),
    }
    .map_err(|_| work_limit_denial(relation.result_path()))?;
    work.charge_adjacency(
        read.adjacency_lists_read(),
        read.relation_records_examined(),
        read.endpoint_records_reserved(),
        relation.result_path(),
    )?;
    let mut targets = BTreeMap::<EntityId, Vec<EntityId>>::new();
    for record in read.into_records() {
        let (parent, child) = match relation.direction() {
            ApplicationQueryResultTraversalDirection::Forward => (record.source, record.target),
            ApplicationQueryResultTraversalDirection::Reverse => (record.target, record.source),
        };
        targets.entry(parent).or_default().push(child);
    }
    let predicate_counts = if relation.predicate().is_some() {
        parents
            .iter()
            .map(|parent| targets.get(&parent.entity_id()).map_or(0, Vec::len))
            .collect::<Vec<_>>()
    } else {
        vec![0; parents.len()]
    };
    let predicate_sources = if relation.predicate().is_some() {
        parents
            .iter()
            .flat_map(|parent| {
                targets
                    .get(&parent.entity_id())
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    for target in targets.values_mut() {
        retain_matching_targets(
            projection, graph, governance, parameters, relation, target, work,
        )?;
    }
    let counts = parents
        .iter()
        .map(|parent| {
            let count = targets.get(&parent.entity_id()).map_or(0, Vec::len);
            validate_relation_cardinality(relation, count)?;
            Ok(count)
        })
        .collect::<Result<Vec<_>, WorthQueryApplicationReadExecutionDenial>>()?;
    let child_ids = parents
        .iter()
        .flat_map(|parent| {
            targets
                .get(&parent.entity_id())
                .into_iter()
                .flat_map(|ids| ids.iter().copied())
        })
        .collect::<Vec<_>>();
    let children = project_nodes(
        runtime,
        projection,
        graph,
        contract,
        governance,
        parameters,
        Some(relation.result_path_identity()),
        relation.result_path(),
        relation.child_entity(),
        &child_ids,
        work,
        collection_selection,
        result_buffer,
    )?;
    distribute_relation_rows(
        parents,
        relation,
        counts,
        children,
        contract,
        governance,
        work,
        false,
        result_buffer,
        predicate_counts,
        predicate_sources,
    )
}

#[allow(clippy::too_many_arguments)]
fn attach_targeted_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    target: &TargetedCollectionChild,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let [parent] = parents else {
        return Err(traversal_denial(relation.result_path()));
    };
    let layout = graph
        .relation(relation.relation())
        .ok_or_else(|| traversal_denial(relation.result_path()))?;
    let frontier = BTreeSet::from([target.child_entity_id]);
    let read = match relation.direction() {
        ApplicationQueryResultTraversalDirection::Forward => projection
            .bounded_incoming_relations_for_frontier(&frontier, layout.kind, work.remaining_work()),
        ApplicationQueryResultTraversalDirection::Reverse => projection
            .bounded_outgoing_relations_for_frontier(&frontier, layout.kind, work.remaining_work()),
    }
    .map_err(|_| work_limit_denial(relation.result_path()))?;
    work.charge_adjacency(
        read.adjacency_lists_read(),
        read.relation_records_examined(),
        read.endpoint_records_reserved(),
        relation.result_path(),
    )?;
    let matching_memberships = read
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
        projection,
        graph,
        governance,
        parameters,
        relation,
        &mut filtered_target,
        work,
    )?;
    if filtered_target.len() != 1 {
        return Err(traversal_denial(relation.result_path()));
    }
    let mut nested_selection = ActiveResultTreeCollectionSelection::Complete;
    let children = project_nodes(
        runtime,
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
        result_buffer,
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
        result_buffer,
        vec![usize::from(relation.predicate().is_some())],
        relation
            .predicate()
            .map(|_| vec![target.child_entity_id])
            .unwrap_or_default(),
    )
}

fn validate_relation_cardinality(
    relation: &WorthQueryInstalledGraphRelation,
    count: usize,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let valid = match relation.cardinality() {
        ApplicationQueryCardinality::OptionalOne => count <= 1,
        ApplicationQueryCardinality::ExactlyOne => count == 1,
        ApplicationQueryCardinality::Many => true,
    };
    if valid {
        Ok(())
    } else {
        Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch,
            relation.result_path(),
        ))
    }
}

fn traversal_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
        subject,
    )
}

fn work_limit_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
        subject,
    )
}
