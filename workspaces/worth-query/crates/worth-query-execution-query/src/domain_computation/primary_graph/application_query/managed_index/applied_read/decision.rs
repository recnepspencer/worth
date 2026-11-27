//! Project managed decision results at the exact candidate that supplied the Source.
use super::{
    parent::select_parent, ApplicationQueryNativeAccess, Denial, SourceFactsAllocation,
    SourceQueryReaderAccess,
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_query::{
        authorized_read::execute_selected_authorized_body_with_native_access,
        observed_source::{fact_conversion, WorthQueryObservedSourceFootprint},
        one_shot::map_selected_read_stop,
        output_source_observation::SourceQueryObservationStop,
        read_execution::{applied_source, read_plan::ReadPlan, ReadInterruption},
        WorthQueryApplicationProjection,
    },
    value_decode::ValueDecodeState,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use super::super::{
    admitted_read::DecisionSource,
    receiver::{BoundPlan, ResultValue},
    AppliedManagedIndexRead,
};

pub(in super::super) fn read_decision<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    plan: BoundPlan<'_, S, B>,
    applied: AppliedManagedIndexRead<'_>,
    association: WorthQueryInstalledManagedIndexParent,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<(Vec<ResultValue<S, B>>, DecisionSource), Denial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    super::own_outputs::require_scope(applied.own, plan.scope.entity_id())?;
    let lease = select_parent::<S, B>(
        execution,
        access,
        &plan,
        applied.before,
        applied.candidate,
        applied.assembly,
        association,
        allocation,
        native,
    )?;
    let request = plan.controls.request_scope();
    let identity = applied.candidate.candidate_identity();
    let rows = lease.rows(&plan.graph_work, request, identity)?;
    let application = access.application;
    let (projected, _, proof) = execute_selected_authorized_body_with_native_access(
        execution,
        application,
        &plan,
        native,
        |_, layout, admitted| {
            let mut read = ReadPlan::of(admitted);
            read.applied_managed_rows = Some(&rows);
            read.material_source = Some(allocation);
            let interruption = ReadInterruption::Query(request);
            let result = (|| {
                let (selection, tree) = applied_source::read_applied_tree(
                    execution,
                    applied.candidate,
                    layout,
                    &read,
                    read.contract.cardinality(),
                    interruption,
                )?;
                let values = applied_source::decode_applied_rows::<S, B::Query, ResultValue<S, B>>(
                    execution,
                    &ValueDecodeState::new(application.installed_schema.value_bindings().clone()),
                    &tree.rows,
                    &read,
                    interruption,
                )?;
                Ok::<_, applied_source::AppliedSourceReadStop>((values, selection, tree))
            })();
            Ok(result)
        },
    )
    .map_err(|cause| {
        SourceQueryObservationStop::Read(map_selected_read_stop(cause, plan.query.name()))
    })?;
    let (values, selection, tree) =
        projected.map_err(|cause| cause.into_source_query_stop(plan.query.name()))?;
    lease.rows(&plan.graph_work, request, identity)?;
    let support = lease.retain_support(execution)?;

    let graph = application
        .runtime
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let control =
        fact_conversion::SourceFactExpansionControl::in_request(*execution, Some(request));
    let facts = retain_decision_facts(
        execution,
        &plan,
        applied,
        graph.layout_ref(),
        selection
            .result_set_source
            .ok_or(Denial::ForeignAssociation)?,
        tree.source_footprints,
        &support,
        allocation,
        control,
    )?;
    // A successor round never sees a row its own chain's effect produced.
    super::own_outputs::require_facts(execution, applied.own, &facts)?;
    let authorization = Arc::clone(support.authorization());
    plan.graph_work
        .complete_query_read(
            proof,
            crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
                examined_candidates: selection.examined_candidates,
                projected_records: tree.projected_records,
                projected_fields: tree.projected_fields,
                relation_records_examined: tree.reads.adjacency_edges_inspected(),
                ordering_comparisons: tree.ordering_comparisons,
            },
            plan.basis.release(),
        )
        .map_err(Denial::GraphRead)?;
    Ok((
        values,
        DecisionSource::Applied {
            facts,
            authorization,
            allocation: Arc::clone(allocation),
        },
    ))
}

/// Retain empty-selection support, Parent projection support and every Native
/// Source coordinate that could change the provider selection.
fn retain_decision_facts<S, Q, P, R, A, I, C>(
    execution: &ExecutionRequest<'_, '_>,
    plan: &crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    applied: AppliedManagedIndexRead<'_>,
    layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    selection: Arc<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection>,
    rows: Vec<WorthQueryObservedSourceFootprint>,
    support: &super::AppliedManagedSourceSupport,
    allocation: &SourceFactsAllocation,
    control: fact_conversion::SourceFactExpansionControl<'_, '_>,
) -> Result<Vec<Fact>, Denial> {
    let scope = plan.scope.entity_id();
    let result_set = WorthQueryObservedSourceFootprint {
        root: scope,
        complete: true,
        entities: Vec::new(),
        aspects: Vec::new(),
        adjacencies: Vec::new(),
        member_groups: Vec::new(),
        root_selection: Some(selection),
    };
    let mut facts = Vec::new();
    for footprint in std::iter::once(result_set).chain(rows.into_iter().map(|mut row| {
        // The selection is captured once, including when no rows matched.
        row.root_selection = None;
        row
    })) {
        let projected =
            fact_conversion::applied_facts(&footprint, layout, plan.query.name(), control)
                .map_err(SourceQueryObservationStop::from)?;
        reserve_facts(execution, allocation, &mut facts, projected.len())?;
        facts.extend(projected);
    }

    let key = plan.scope.identity_locator();
    let locator = worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        key.aspect().aspect_key().clone(),
        key.field_path().clone(),
    );
    let native_revision = applied
        .candidate
        .project_candidate()
        .entity_field_revision(execution, scope, &locator)?;
    reserve_facts(
        execution,
        allocation,
        &mut facts,
        support
            .facts()
            .len()
            .checked_add(1)
            .ok_or(Cause::ChargedBytesOverflow)?,
    )?;
    facts.push(Fact::SourceFieldRevision {
        entity_id: scope,
        locator,
        native_revision,
    });
    for fact in support.facts() {
        SourceFactsAllocation::work(execution, 1)?;
        facts.push(fact.clone());
    }
    Ok(facts)
}

fn reserve_facts(
    execution: &ExecutionRequest<'_, '_>,
    allocation: &SourceFactsAllocation,
    facts: &mut Vec<Fact>,
    additional: usize,
) -> Result<(), Cause> {
    let needed = facts
        .len()
        .checked_add(additional)
        .ok_or(Cause::ChargedBytesOverflow)?;
    let prior = facts.capacity();
    if needed > prior {
        let target = needed.max(prior.saturating_mul(2));
        allocation.claim_slice::<Fact>(execution, target - prior)?;
        facts
            .try_reserve_exact(target - facts.len())
            .map_err(|_| Cause::AllocationFailed)?;
    }
    Ok(())
}
