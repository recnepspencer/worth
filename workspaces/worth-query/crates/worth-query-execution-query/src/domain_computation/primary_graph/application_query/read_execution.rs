use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
pub mod applied_source;
mod cardinality;
pub mod denial;
pub mod dispatch_witness;
use cardinality::{validate_cardinality_and_limit, validate_read_cardinality};
pub mod entry_observation;
pub mod interruption;
pub use interruption::ReadInterruption;
mod kernel_capacity;
mod kernel_outcome;
pub mod live_target;
mod pinning;
pub mod prepared_pair;
pub mod read_plan;
use pinning::pin_runtime_read;
mod root_selection;
pub mod tree_materialization;
pub mod work_observation;

pub use entry_observation::{
    query_read_kernel_entries_by_root_on_this_thread_for_test,
    query_read_kernel_entries_on_this_thread_for_test,
};
pub use work_observation::OneShotReadWorkObservation;

use super::{
    projection::WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryAdmittedApplicationQueryPlan,
};
pub use denial::{managed_read_denial, read_execution_denial, relational_read_denial};
pub use denial::{
    WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
};
pub use kernel_outcome::{
    complete_single_source_selection, project_non_live_kernel_roots,
    project_non_live_kernel_source, project_non_live_kernel_with_decode,
    NonLiveKernelReceiptEvidence, ProjectedNonLiveKernelOutcome,
};
pub use live_target::read_live_target;
use root_selection::select_bounded_roots;
pub use tree_materialization::fallible_heap;
use tree_materialization::materialize_result_tree;
use tree_materialization::{OrderedCollectionWindow, ResultTreeCollectionSelection};
use worth_relational::facade::indexes::{DerivedIndexGenerationId, RelatedEntityOrderingBoundary};

pub struct RawOneShotRows {
    pub(crate) material_source: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    >,
    pub(crate) rows: WorthQueryApplicationDisclosedProjectionTree,
    pub(crate) source_footprints: Vec<super::observed_source::WorthQueryObservedSourceFootprint>,
    pub(crate) result_set_source:
        Option<std::sync::Arc<super::observed_source::WorthQueryObservedRootSelection>>,
    pub(crate) examined_candidates: usize,
    pub(crate) predicate_work_units: usize,
    pub(crate) predicate_index_generation:
        Option<worth_relational::facade::indexes::DerivedIndexGenerationId>,
    pub(crate) target_identity_index_generation: Option<DerivedIndexGenerationId>,
    pub(crate) projected_records: usize,
    pub(crate) projected_fields: usize,
    pub(crate) reads: WorthQueryInvariantProjectionWork,
    pub(crate) root_path_steps: Option<std::sync::Arc<super::access_receipt::RootPathReadEvidence>>,
    pub(crate) ordering_comparisons: usize,
    pub(crate) ordered_index_generation: Option<DerivedIndexGenerationId>,
    pub(crate) ordered_index_entries_examined: usize,
    pub(crate) next_boundary: Option<RelatedEntityOrderingBoundary>,
    pub(crate) has_more: bool,
    pub(crate) actual_work: usize,
}

pub struct RawNonLiveKernelOutcome {
    pub(crate) raw: RawOneShotRows,
    /// The persistent names of the materialized rows' records, when the
    /// read's projection asked for them and the name index answered.
    pub(crate) persistent_names: Option<worth_query_execution_naming::facade::ObservedEntityNames>,
    /// What the projected names rest on beyond the rows' own records: each
    /// named row member's group membership (`persistent_names`).
    pub(crate) persistent_name_facts:
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
}

pub struct RawLiveKernelOutcome {
    pub(crate) raw: RawOneShotRows,
}

pub fn read_bounded_root_rows<
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
    read_bounded_plan_rows(
        runtime,
        execution,
        graph,
        &read,
        spent,
        ReadInterruption::Query(plan.controls.request_scope()),
    )
}

pub fn read_bounded_plan_rows(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    read: &read_plan::ReadPlan<'_>,
    spent: Option<&OneShotReadWorkObservation<'_>>,
    interruption: ReadInterruption<'_>,
) -> Result<RawNonLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    let pinned = pin_runtime_read(
        runtime,
        execution,
        read,
        &[],
        WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
    )?;
    read_prepared_root_rows(&pinned, execution, graph, read, spent, interruption)
}

pub fn read_prepared_root_rows(
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
    let window = match plan.root_window {
        Some(window) => Some(select_root_window_roots(
            runtime,
            execution,
            graph,
            plan,
            &selection.candidates,
            window,
            spent,
            interruption,
        )?),
        None => {
            validate_read_cardinality(contract.cardinality(), selection.candidates.len(), plan)?;
            None
        }
    };
    let roots = window
        .as_ref()
        .map_or(selection.candidates.as_slice(), |window| &window.roots);
    let tree = materialize_result_tree(
        execution,
        runtime,
        plan.snapshot,
        graph,
        contract,
        plan.governance,
        plan.parameters,
        roots,
        &selection.selected_predicate_sources,
        selection.root_path_source.as_ref(),
        ResultTreeCollectionSelection::Complete,
        interruption,
        spent,
        plan.material_source,
    )?;
    if let Some(components) = selection
        .result_set_source
        .as_ref()
        .and_then(|source| source.components.as_ref())
        && !components.matches_rows(tree.source_footprints.iter().map(|row| row.root))
    {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
            plan.name,
        ));
    }
    let (window_reads, window_comparisons, window_work) = window.map_or_else(
        || (WorthQueryInvariantProjectionWork::default(), 0, 0),
        |window| (window.reads, window.ordering_comparisons, window.work_units),
    );
    let actual_work = selection
        .work_units
        .saturating_add(tree.work_units)
        .saturating_add(window_work);
    Ok(RawNonLiveKernelOutcome {
        raw: RawOneShotRows {
            material_source: plan.material_source.cloned(),
            rows: tree.rows,
            source_footprints: tree.source_footprints,
            result_set_source: selection.result_set_source,
            examined_candidates: selection.examined_candidates,
            predicate_work_units: selection
                .predicate_work_units
                .saturating_add(tree.relation_predicate_work_units),
            predicate_index_generation: selection.predicate_index_generation,
            target_identity_index_generation: None,
            projected_records: tree.projected_records,
            projected_fields: tree.projected_fields,
            reads: selection.reads.combined(tree.reads).combined(window_reads),
            root_path_steps: selection.root_path_steps,
            ordering_comparisons: tree.ordering_comparisons.saturating_add(window_comparisons),
            ordered_index_generation: None,
            ordered_index_entries_examined: 0,
            next_boundary: None,
            has_more: false,
            actual_work,
        },
        persistent_names: None,
        persistent_name_facts: Vec::new(),
    })
}

/// A window pages a `Many` collection, never an exact row, and is at most as
/// wide as the result ceiling.
fn select_root_window_roots(
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    plan: &read_plan::ReadPlan<'_>,
    candidates: &[worth_relational::facade::identity::EntityId],
    window: &super::root_window::WorthQueryApplicationRootWindow,
    spent: Option<&OneShotReadWorkObservation<'_>>,
    interruption: ReadInterruption<'_>,
) -> Result<
    tree_materialization::root_window::RootWindowSelection,
    WorthQueryApplicationReadExecutionDenial,
> {
    // Orbit (anchor-forward) and component (whole-set metadata) selections
    // are not windowable; the request layer refuses them first.
    if plan.contract.orbit().is_some() || plan.contract.component_selection().is_some() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TraversalUnavailable,
            plan.name,
        ));
    }
    if plan.contract.cardinality() != ApplicationQueryCardinality::Many {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch,
            plan.name,
        ));
    }
    if window.width().get() > plan.maximum_result_count {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded,
            plan.name,
        ));
    }
    tree_materialization::root_window::select_root_window(
        execution,
        runtime,
        plan.snapshot,
        graph,
        plan.contract,
        plan.governance,
        candidates,
        window,
        interruption,
        spent,
    )
}

pub mod continuation_page;
pub mod persistent_names;
pub use continuation_page::read_continuation_page;

pub use tree_materialization::validate_relation_cardinality;

mod court_doors_for_test;
