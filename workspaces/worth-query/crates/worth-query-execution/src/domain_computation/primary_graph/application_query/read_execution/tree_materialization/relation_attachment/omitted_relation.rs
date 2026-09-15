use worth_query_installation::facade::WorthQueryInstalledGraphRelation;
use worth_relational::facade::indexes::{
    BoundedIndexParityMode, BoundedRelatedEntityOrderedLookupRequest,
};

use super::{ordered::ordered_lookup_denial, traversal_denial};
use crate::domain_computation::primary_graph::application_query::read_execution::tree_materialization::{
    ActiveResultTreeCollectionSelection, OrderedCollectionProgress, ResultTreeWork,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};

pub(in crate::domain_computation::primary_graph::application_query::read_execution::tree_materialization) fn advance_omitted_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    relation: &WorthQueryInstalledGraphRelation,
    parents: &[WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    collection_selection: &mut ActiveResultTreeCollectionSelection,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let ActiveResultTreeCollectionSelection::Ordered(window) = collection_selection else {
        return Ok(());
    };
    if window
        .request
        .as_ref()
        .is_none_or(|request| request.collection_path != relation.result_path())
    {
        return Ok(());
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
    let generation_id = page.generation_id();
    let has_more = page.has_more();
    let next_boundary = page.into_next_boundary();
    window.progress = Some(OrderedCollectionProgress {
        generation_id,
        next_boundary,
        has_more,
    });
    Ok(())
}
