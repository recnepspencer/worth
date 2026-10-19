use super::*;

pub(in crate::domain_computation::primary_graph::application_query) fn read_continuation_page<
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
    let pinned = pin_runtime_read(
        runtime,
        execution,
        graph,
        &read_plan::ReadPlan::of(plan),
        &[index_id],
        WorthQueryApplicationReadExecutionDenialKind::ContinuationIndexUnavailable,
    )?;
    let runtime = &pinned;
    let selection = select_bounded_roots(
        runtime,
        execution,
        graph,
        &read_plan::ReadPlan::of(plan),
        &mut result_buffer,
        false,
        None,
        plan.controls
            .maximum_work()
            .map(std::num::NonZeroUsize::get),
        ReadInterruption::Query(plan.controls.request_scope()),
    )?;
    validate_cardinality_and_limit(contract.cardinality(), selection.candidates.len(), plan)?;
    let tree = materialize_result_tree(
        execution,
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
            .map(|maximum| maximum.get().saturating_sub(selection.work_units)),
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
        ReadInterruption::Query(plan.controls.request_scope()),
        &mut result_buffer,
        None,
    )?;
    let actual_work = selection.work_units.saturating_add(tree.work_units);
    if plan
        .controls
        .maximum_work()
        .is_some_and(|maximum| actual_work > maximum.get())
    {
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
