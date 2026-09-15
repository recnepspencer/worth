use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryLiveTargetMode;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode, DerivedIndexGenerationId,
};

use super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::{
    read_execution_denial, root_selection::select_bounded_roots,
    tree_materialization::materialize_result_tree,
    tree_materialization::ResultTreeCollectionSelection, RawLiveKernelOutcome, RawOneShotRows,
    WorthQueryApplicationReadExecutionDenial, WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationResultBufferReservation;

pub(in crate::domain_computation::primary_graph::application_query) fn read_live_target<
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
    target_identity: AspectValue,
    mut result_buffer: WorthQueryApplicationResultBufferReservation,
) -> Result<RawLiveKernelOutcome, WorthQueryApplicationReadExecutionDenial> {
    let contract = plan.query.read_family_binding().planning_contract();
    let live = plan.query.live().ok_or_else(|| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
            plan.query.name(),
        )
    })?;
    let selection = select_bounded_roots(runtime, graph, plan)?;
    super::validate_cardinality_and_limit(
        contract.cardinality(),
        selection.candidates.len(),
        plan,
    )?;
    let target = resolve_live_target(runtime, graph, plan, live, target_identity)?;
    let target_lookup_work = target.examined_entry_count;
    let collection_selection = match live.target_mode() {
        ApplicationQueryLiveTargetMode::Root => {
            if selection.candidates.as_slice() != [target.entity_id] {
                return Err(read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound,
                    live.target_identity().result_path(),
                ));
            }
            ResultTreeCollectionSelection::Complete
        }
        ApplicationQueryLiveTargetMode::Collection => {
            let collection_path = live.collection_path().ok_or_else(|| {
                read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
                    plan.query.name(),
                )
            })?;
            ResultTreeCollectionSelection::Targeted(
                super::tree_materialization::TargetedCollectionChild {
                    collection_path: collection_path.to_string(),
                    child_entity_id: target.entity_id,
                },
            )
        }
    };
    let admitted_before_materialization = selection.work_units.saturating_add(target_lookup_work);
    let tree = materialize_result_tree(
        runtime,
        plan.basis.snapshot_handle(),
        graph,
        contract,
        &plan.governance,
        &plan.parameters,
        &selection.candidates,
        plan.controls
            .maximum_work()
            .get()
            .saturating_sub(admitted_before_materialization),
        collection_selection,
        &mut result_buffer,
    )?;
    let actual_work = admitted_before_materialization.saturating_add(tree.work_units);
    if actual_work > plan.controls.maximum_work().get() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            plan.query.name(),
        ));
    }
    super::verify_result_tree_accounting(
        &result_buffer,
        tree.rows.raw_rows(),
        tree.rows.capacity(),
        &tree.source_footprints,
        tree.source_footprints.capacity(),
        plan.query.name(),
    )?;
    Ok(RawLiveKernelOutcome {
        raw: RawOneShotRows {
            rows: tree.rows,
            source_footprints: tree.source_footprints,
            examined_candidates: selection
                .examined_candidates
                .saturating_add(target_lookup_work),
            predicate_work_units: selection
                .predicate_work_units
                .saturating_add(target_lookup_work)
                .saturating_add(tree.relation_predicate_work_units),
            predicate_index_generation: selection.predicate_index_generation,
            target_identity_index_generation: target.generation_id,
            target_identity_index_entries_examined: target_lookup_work,
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

struct ResolvedLiveTarget {
    entity_id: worth_relational::facade::identity::EntityId,
    examined_entry_count: usize,
    generation_id: Option<DerivedIndexGenerationId>,
}

fn resolve_live_target<
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
    live: &worth_query_installation::facade::WorthQueryInstalledApplicationLiveContract,
    target_identity: AspectValue,
) -> Result<ResolvedLiveTarget, WorthQueryApplicationReadExecutionDenial> {
    let target = live.target_identity();
    let computation = plan
        .governance
        .admit_internal_projection(
            (target.entity(), target.aspect(), target.field()),
            target.field_key(),
            ApplicationQueryObservableInfluence::LiveMembership,
        )
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
                target.result_path(),
            )
        })?;
    let layout = graph
        .equality_field(target.entity(), target.aspect(), target.field())
        .ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
                target.result_path(),
            )
        })?;
    if !computation.admits_locator(&layout.locator) {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
            target.result_path(),
        ));
    }
    let request = BoundedEntityFieldLookupRequest::new(
        plan.basis.snapshot_handle().clone(),
        layout.equality_index_id.ok_or_else(|| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
                target.result_path(),
            )
        })?,
        layout.entity_kind,
        layout.locator.clone(),
        target_identity,
        2,
    )
    .map_err(|_| {
        read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
            target.result_path(),
        )
    })?;
    let lookup =
        execute_governed_live_target_lookup(runtime, computation, request).map_err(|_| {
            read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::TargetIdentityIndexUnavailable,
                target.result_path(),
            )
        })?;
    if lookup.overflowed() {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TargetIdentityLookupOverflow,
            target.result_path(),
        ));
    }
    let [entity_id] = lookup.candidate_entity_ids() else {
        return Err(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::TargetIdentityNotFound,
            target.result_path(),
        ));
    };
    Ok(ResolvedLiveTarget {
        entity_id: *entity_id,
        examined_entry_count: lookup.examined_entry_count(),
        generation_id: Some(lookup.generation_id()),
    })
}

fn execute_governed_live_target_lookup(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    _projection: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationInternalProjectionAdmission<'_>,
    request: BoundedEntityFieldLookupRequest,
) -> Result<
    worth_relational::facade::indexes::BoundedEntityFieldLookupOutcome,
    worth_relational::facade::indexes::BoundedEntityFieldLookupDenial,
> {
    runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
}
