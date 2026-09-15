use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupDenialKind,
    BoundedRelatedEntityOrderedLookupRequest,
};

use super::super::relation_distribution::distribute_relation_rows;
use super::super::{
    project_nodes, ActiveOrderedCollectionWindow, ActiveResultTreeCollectionSelection,
    OrderedCollectionProgress, ResultTreeWork, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::{
    read_execution::{read_execution_denial, WorthQueryApplicationReadExecutionDenialKind},
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn attach_ordered_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    window: &mut ActiveOrderedCollectionWindow,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if relation.predicate().is_some() {
        return Err(traversal_denial(relation.result_path()));
    }
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
        parameters,
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
        vec![0],
        Vec::new(),
    )
}

fn traversal_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
        subject,
    )
}

pub(super) fn ordered_lookup_denial(
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
