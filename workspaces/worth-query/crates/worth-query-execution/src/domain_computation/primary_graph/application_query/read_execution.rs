use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
mod cardinality;
mod denial;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) mod dispatch_witness;
use cardinality::{validate_cardinality_and_limit, validate_read_cardinality};
mod entry_observation;
mod interruption;
use interruption::ReadInterruption;
mod kernel_capacity;
mod kernel_outcome;
mod live_target;
mod pinning;
pub(super) mod prepared_pair;
pub(super) mod read_plan;
use pinning::pin_runtime_read;
mod root_selection;
mod tree_materialization;
mod work_observation;
pub use entry_observation::{
    query_read_kernel_entries_by_root_on_this_thread_for_test,
    query_read_kernel_entries_on_this_thread_for_test,
};
pub(in crate::domain_computation::primary_graph::application_query) use work_observation::OneShotReadWorkObservation;

use super::{
    projection::WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryAdmittedApplicationQueryPlan,
};
pub(super) use denial::{managed_read_denial, read_execution_denial, relational_read_denial};
pub(super) use denial::{
    WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
};
pub(super) use kernel_outcome::{
    project_non_live_kernel_roots, project_non_live_kernel_with_decode,
    NonLiveKernelReceiptEvidence, ProjectedNonLiveKernelOutcome,
};
pub(super) use live_target::read_live_target;
use root_selection::select_bounded_roots;
use tree_materialization::materialize_result_tree;
use tree_materialization::{OrderedCollectionWindow, ResultTreeCollectionSelection};
use worth_relational::facade::indexes::{DerivedIndexGenerationId, RelatedEntityOrderingBoundary};

pub(super) struct RawOneShotRows {
    pub(super) rows: WorthQueryApplicationDisclosedProjectionTree,
    pub(super) source_footprints: Vec<super::observed_source::WorthQueryObservedSourceFootprint>,
    pub(super) result_set_source:
        Option<std::sync::Arc<super::observed_source::WorthQueryObservedRootSelection>>,
    pub(super) examined_candidates: usize,
    pub(super) predicate_work_units: usize,
    pub(super) predicate_index_generation:
        Option<worth_relational::facade::indexes::DerivedIndexGenerationId>,
    pub(super) target_identity_index_generation: Option<DerivedIndexGenerationId>,
    pub(super) target_identity_index_entries_examined: usize,
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
    pub(super) ordering_comparisons: usize,
    pub(super) ordered_index_generation: Option<DerivedIndexGenerationId>,
    pub(super) ordered_index_entries_examined: usize,
    pub(super) next_boundary: Option<RelatedEntityOrderingBoundary>,
    pub(super) has_more: bool,
    pub(super) actual_work: usize,
}

pub(super) struct RawNonLiveKernelOutcome {
    pub(super) raw: RawOneShotRows,
}

pub(super) struct RawLiveKernelOutcome {
    pub(super) raw: RawOneShotRows,
}

pub(super) fn read_bounded_root_rows<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,

    spent: Option<&OneShotReadWorkObservation<'_>>,
) -> Result<RawNonLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    let read = read_plan::ReadPlan::of(plan);
    let pinned = pin_runtime_read(
        runtime,
        execution,
        &read,
        &[],
        WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
    )?;
    read_prepared_root_rows(
        &pinned,
        execution,
        graph,
        &read_plan::ReadPlan::of(plan),
        spent,
        ReadInterruption::Query(plan.controls.request_scope()),
    )
}

pub(super) fn read_prepared_root_rows(
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &read_plan::ReadPlan<'_>,

    spent: Option<&OneShotReadWorkObservation<'_>>,

    interruption: ReadInterruption<'_>,
) -> Result<RawNonLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    entry_observation::record_kernel_entry(plan.root);
    let contract = plan.contract;
    let selection =
        select_bounded_roots(runtime, execution, graph, plan, true, spent, interruption)?;
    validate_read_cardinality(contract.cardinality(), selection.candidates.len(), plan)?;
    let tree = materialize_result_tree(
        execution,
        runtime,
        plan.snapshot,
        graph,
        contract,
        plan.governance,
        plan.parameters,
        &selection.candidates,
        selection.selected_predicate_source.as_ref(),
        selection.root_path_source.as_ref(),
        ResultTreeCollectionSelection::Complete,
        interruption,
        spent,
    )?;
    let actual_work = selection.work_units.saturating_add(tree.work_units);
    Ok(RawNonLiveKernelOutcome {
        raw: RawOneShotRows {
            rows: tree.rows,
            source_footprints: tree.source_footprints,
            result_set_source: selection.result_set_source,
            examined_candidates: selection.examined_candidates,
            predicate_work_units: selection
                .predicate_work_units
                .saturating_add(tree.relation_predicate_work_units),
            predicate_index_generation: selection.predicate_index_generation,
            target_identity_index_generation: None,
            target_identity_index_entries_examined: 0,
            projected_records: tree.projected_records,
            projected_fields: tree.projected_fields,
            adjacency_lists_read: selection
                .adjacency_lists_read
                .saturating_add(tree.adjacency_lists_read),
            relation_records_examined: selection
                .relation_records_examined
                .saturating_add(tree.relation_records_examined),
            ordering_comparisons: tree.ordering_comparisons,
            ordered_index_generation: None,
            ordered_index_entries_examined: 0,
            next_boundary: None,
            has_more: false,
            actual_work,
        },
    })
}

mod continuation_page;
pub(super) use continuation_page::read_continuation_page;
