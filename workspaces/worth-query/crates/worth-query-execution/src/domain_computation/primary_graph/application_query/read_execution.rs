use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
mod denial;
mod kernel_outcome;
mod live_target;
mod root_selection;
mod tree_materialization;

use super::resource_lifecycle::WorthQueryApplicationResultBufferReservation;
use super::{
    projection::{
        WorthQueryApplicationDisclosedProjectionTree, WorthQueryApplicationProjectionNode,
    },
    WorthQueryAdmittedApplicationQueryPlan,
};
pub(super) use denial::read_execution_denial;
pub(super) use denial::{
    WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
};
pub(super) use kernel_outcome::{project_non_live_kernel, NonLiveKernelReceiptEvidence};
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
    pub(super) result_buffer: WorthQueryApplicationResultBufferReservation,
}

pub(super) struct RawLiveKernelOutcome {
    pub(super) raw: RawOneShotRows,
    pub(super) result_buffer: WorthQueryApplicationResultBufferReservation,
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
    mut result_buffer: WorthQueryApplicationResultBufferReservation,
) -> Result<RawNonLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    let contract = plan.query.read_family_binding().planning_contract();
    let selection = select_bounded_roots(runtime, graph, plan, &mut result_buffer, true)?;
    validate_cardinality_and_limit(contract.cardinality(), selection.candidates.len(), plan)?;
    let tree = materialize_result_tree(
        runtime,
        plan.basis.snapshot_handle(),
        graph,
        contract,
        &plan.governance,
        &plan.parameters,
        &selection.candidates,
        selection.selected_predicate_source.as_ref(),
        selection.root_path_source.as_ref(),
        plan.controls
            .maximum_work()
            .get()
            .saturating_sub(selection.work_units),
        ResultTreeCollectionSelection::Complete,
        &mut result_buffer,
    )?;
    let actual_work = selection.work_units.saturating_add(tree.work_units);
    if actual_work > plan.controls.maximum_work().get() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            plan.query.name(),
        ));
    }
    verify_result_tree_accounting(
        &result_buffer,
        tree.rows.raw_rows(),
        tree.rows.capacity(),
        &tree.source_footprints,
        tree.source_footprints.capacity(),
        selection.result_set_source.as_deref(),
        plan.query.name(),
    )?;
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
        result_buffer,
    })
}

pub(super) fn read_continuation_page<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
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
    after: Option<RelatedEntityOrderingBoundary>,
    mut result_buffer: WorthQueryApplicationResultBufferReservation,
) -> Result<RawNonLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    let contract = plan.query.read_family_binding().planning_contract();
    let continuation = plan.query.continuation().ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable,
            plan.query.name(),
        )
    })?;
    let index_id = plan.continuation_index_id.ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable,
            plan.query.name(),
        )
    })?;
    let selection = select_bounded_roots(runtime, graph, plan, &mut result_buffer, false)?;
    validate_cardinality_and_limit(contract.cardinality(), selection.candidates.len(), plan)?;
    let tree = materialize_result_tree(
        runtime,
        plan.basis.snapshot_handle(),
        graph,
        contract,
        &plan.governance,
        &plan.parameters,
        &selection.candidates,
        selection.selected_predicate_source.as_ref(),
        selection.root_path_source.as_ref(),
        plan.controls
            .maximum_work()
            .get()
            .saturating_sub(selection.work_units),
        ResultTreeCollectionSelection::Ordered(OrderedCollectionWindow {
            snapshot: plan.basis.snapshot_handle().clone(),
            collection_path: continuation.collection_path().to_string(),
            index_id,
            expected_generation: plan
                .continuation_state
                .as_ref()
                .map(|state| state.expected_generation),
            after,
            page_width: plan.controls.maximum_result_count().get(),
        }),
        &mut result_buffer,
    )?;
    let actual_work = selection.work_units.saturating_add(tree.work_units);
    if actual_work > plan.controls.maximum_work().get() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            plan.query.name(),
        ));
    }
    let progress = tree.continuation.ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable,
            continuation.collection_path(),
        )
    })?;
    verify_result_tree_accounting(
        &result_buffer,
        tree.rows.raw_rows(),
        tree.rows.capacity(),
        &tree.source_footprints,
        tree.source_footprints.capacity(),
        selection.result_set_source.as_deref(),
        plan.query.name(),
    )?;
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
            ordered_index_generation: Some(progress.generation_id),
            ordered_index_entries_examined: tree.ordered_index_entries_examined,
            next_boundary: progress.next_boundary,
            has_more: progress.has_more,
            actual_work,
        },
        result_buffer,
    })
}

fn verify_result_tree_accounting(
    reservation: &WorthQueryApplicationResultBufferReservation,
    rows: &[WorthQueryApplicationProjectionNode],
    row_capacity: usize,
    source_footprints: &[super::observed_source::WorthQueryObservedSourceFootprint],
    source_footprint_capacity: usize,
    result_set_source: Option<&super::observed_source::WorthQueryObservedRootSelection>,
    subject: &str,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let retained_bytes = rows
        .iter()
        .map(WorthQueryApplicationProjectionNode::retained_bytes)
        .fold(
            row_capacity.saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectionNode>()),
            usize::saturating_add,
        );
    let retained_bytes = source_footprints
        .iter()
        .map(super::observed_source::WorthQueryObservedSourceFootprint::retained_bytes)
        .fold(
            retained_bytes.saturating_add(source_footprint_capacity.saturating_mul(
                std::mem::size_of::<super::observed_source::WorthQueryObservedSourceFootprint>(),
            )),
            usize::saturating_add,
        );
    let retained_bytes = source_footprints
        .iter()
        .fold(retained_bytes, |bytes, footprint| {
            bytes.saturating_add(
                footprint
                    .root_selection
                    .as_ref()
                    .map_or(0, |source| source.retained_bytes()),
            )
        });
    let retained_bytes = retained_bytes
        .saturating_add(result_set_source.map_or(0, |source| source.retained_bytes()));
    reservation.verify_retained(retained_bytes).map_err(|()| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded,
            subject,
        )
    })
}

fn validate_cardinality_and_limit<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    cardinality: ApplicationQueryCardinality,
    count: usize,
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
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    if count > plan.controls.maximum_result_count().get() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ResultLimitExceeded,
            plan.query.name(),
        ));
    }
    let valid = match cardinality {
        ApplicationQueryCardinality::OptionalOne => count <= 1,
        ApplicationQueryCardinality::ExactlyOne => count == 1,
        ApplicationQueryCardinality::Many => true,
    };
    if valid {
        Ok(())
    } else {
        Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch,
            plan.query.name(),
        ))
    }
}
