use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::super::{
    access_receipt::WorthQueryApplicationQueryAccessReceiptParts,
    read_execution::RawLiveKernelOutcome, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationProjectionRow,
    WorthQueryApplicationQueryAccessReceipt,
};

pub(super) enum WorthQueryLiveProjectionFinalizationDenial {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
    BasisRelease,
    ResultShape,
    Projection(WorthQueryApplicationProjectionDenialKind),
}

pub(super) fn finalize_live_projection<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    plan: WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    kernel: RawLiveKernelOutcome,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
) -> Result<
    (
        QueryResult,
        WorthQueryApplicationQueryAccessReceipt,
        super::super::disclosure::WorthQueryApplicationQueryGovernance,
    ),
    WorthQueryLiveProjectionFinalizationDenial,
>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let RawLiveKernelOutcome { raw, result_buffer } = kernel;
    let basis_identity = plan.basis.identity().clone();
    let basis_version = plan.basis.version_id();
    let basis_release = plan.basis.release();
    if let super::super::resource_lifecycle::WorthQueryApplicationBasisReleaseOutcome::Handle(
        handle,
    ) = basis_release.outcome()
    {
        return Err(WorthQueryLiveProjectionFinalizationDenial::Handle(handle));
    }
    let released = basis_release.released();
    if !released {
        return Err(WorthQueryLiveProjectionFinalizationDenial::BasisRelease);
    }
    let Some(node) = raw.rows.only() else {
        return Err(WorthQueryLiveProjectionFinalizationDenial::ResultShape);
    };
    let result = QueryResult::project(&WorthQueryApplicationProjectionRow::new(
        node,
        &plan.governance,
    ))
    .map_err(|denial| WorthQueryLiveProjectionFinalizationDenial::Projection(denial.kind()))?;
    drop(raw.rows);
    let result_buffer = result_buffer.release();
    let read_completion = plan
        .graph_work
        .complete_query_read(
            read_proof,
            crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
                examined_candidates: raw.examined_candidates,
                projected_records: raw.projected_records,
                projected_fields: raw.projected_fields,
                relation_records_examined: raw.relation_records_examined,
                ordering_comparisons: raw.ordering_comparisons,
            },
            basis_release,
        )
        .map_err(|_| WorthQueryLiveProjectionFinalizationDenial::ResultShape)?;
    let receipt = WorthQueryApplicationQueryAccessReceipt::new(
        WorthQueryApplicationQueryAccessReceiptParts {
            query_identity: plan.query.identity().clone(),
            parameter_binding_identity: *plan.parameters.identity(),
            graph_authority_identity: plan.graph_authority_identity,
            provider_identity: plan.provider_identity,
            basis_identity,
            basis_version,
            basis_posture: plan.controls.basis_posture(),
            lane: plan.controls.lane(),
            consistency: plan.controls.consistency(),
            freshness: plan.controls.freshness(),
            predicate_index_generation: raw.predicate_index_generation,
            target_identity_index_generation: raw.target_identity_index_generation,
            ordered_index_generation: raw.ordered_index_generation,
            read_completion,
            canonical_work: plan.canonical_work,
            authorization_work,
            examined_candidate_count: raw.examined_candidates,
            predicate_work_units: raw.predicate_work_units,
            projected_record_count: raw.projected_records,
            projected_field_count: raw.projected_fields,
            adjacency_list_read_count: raw.adjacency_lists_read,
            edge_scan_count: raw.relation_records_examined,
            ordering_comparison_count: raw.ordering_comparisons,
            ordered_index_entry_count: raw.ordered_index_entries_examined,
            target_identity_index_entry_count: raw.target_identity_index_entries_examined,
            per_result_neighbor_lookup_count: 0,
            fallback_count: 0,
            result_count: 1,
            truncation_count: 0,
            total_work_units: raw.actual_work,
            result_buffer: Some(result_buffer),
            basis_released: released,
            disclosure: plan.governance.receipt(),
        },
    );
    Ok((result, receipt, plan.governance))
}
