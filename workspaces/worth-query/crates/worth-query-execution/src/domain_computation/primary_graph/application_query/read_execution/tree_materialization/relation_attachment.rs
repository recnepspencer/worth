use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryResultTraversalDirection,
};
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::{
    identity::EntityId,
    indexes::{
        BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenialKind,
        BoundedRelatedEntityOrderedLookupRequest,
    },
};

use super::relation_distribution::distribute_relation_rows;
use super::{
    project_nodes, ActiveOrderedCollectionWindow, ActiveResultTreeCollectionSelection,
    OrderedCollectionProgress, ResultTreeWork, TargetedCollectionChild,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::{
    read_execution::{read_execution_denial, WorthQueryApplicationReadExecutionDenialKind},
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};

mod omitted_relation;
pub(super) use omitted_relation::advance_omitted_relation;

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
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
    )
}

#[allow(clippy::too_many_arguments)]
fn attach_ordered_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    window: &mut ActiveOrderedCollectionWindow,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let request = window
        .request
        .take()
        .ok_or_else(|| traversal_denial(relation.result_path()))?;
    let [parent] = parents else {
        return Err(traversal_denial(relation.result_path()));
    };
    let child_kind = graph
        .entity_kind(relation.child_entity())
        .ok_or_else(|| traversal_denial(relation.result_path()))?;
    let mut lookup = BoundedRelatedEntityOrderedLookupRequest::new(
        request.snapshot,
        request.index_id,
        parent.entity_id(),
        child_kind,
        request.after,
        request.page_width,
    )
    .map_err(|denial| ordered_lookup_denial(denial.kind(), relation.result_path()))?;
    if let Some(expected_generation) = request.expected_generation {
        lookup = lookup.expect_generation(expected_generation);
    }
    let page = runtime
        .index_access()
        .execute_bounded_related_entity_ordered_lookup(lookup, BoundedIndexParityMode::Production)
        .map_err(|denial| ordered_lookup_denial(denial.kind(), relation.result_path()))?;
    work.charge_ordered_index_entries(page.examined_entry_count(), relation.result_path())?;
    let child_ids = page.child_entity_ids().to_vec();
    let mut nested_selection = ActiveResultTreeCollectionSelection::Complete;
    let children = project_nodes(
        runtime,
        projection,
        graph,
        contract,
        governance,
        Some(relation.result_path_identity()),
        relation.result_path(),
        relation.child_entity(),
        &child_ids,
        work,
        &mut nested_selection,
        result_buffer,
    )?;
    let has_more = page.has_more();
    let generation_id = page.generation_id();
    let next_boundary = page.into_next_boundary();
    window.progress = Some(OrderedCollectionProgress {
        generation_id,
        next_boundary,
        has_more,
    });
    distribute_relation_rows(
        parents,
        relation,
        vec![children.len()],
        children,
        contract,
        governance,
        work,
        true,
        result_buffer,
    )
}

#[allow(clippy::too_many_arguments)]
fn attach_targeted_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
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
    let mut nested_selection = ActiveResultTreeCollectionSelection::Complete;
    let children = project_nodes(
        runtime,
        projection,
        graph,
        contract,
        governance,
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

fn ordered_lookup_denial(
    kind: BoundedRelatedEntityOrderedLookupDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationReadExecutionDenial {
    let kind = match kind {
        BoundedRelatedEntityOrderedLookupDenialKind::InvalidPageWidth => {
            WorthQueryApplicationReadExecutionDenialKind::ContinuationPageWidthInvalid
        }
        BoundedRelatedEntityOrderedLookupDenialKind::ForeignBoundary => {
            WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected
        }
        BoundedRelatedEntityOrderedLookupDenialKind::ExpectedGenerationMismatch => {
            WorthQueryApplicationReadExecutionDenialKind::ContinuationGenerationChanged
        }
        _ => WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable,
    };
    read_execution_denial(kind, subject)
}
