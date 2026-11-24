//! Complete conservative scope candidates from actual authored and physical inputs.
use super::super::applied::context::SourcePreparationContext;
use super::super::applied::discovery::DiscoveredSource;
use super::{
    admission, collection::AuthoredSourceInputs, ManagedSourceImpactKey, ManagedSourceImpactRows,
    PreparedSourceImpactHistory,
};
use crate::derived_relation::read_denial::DerivedRelationReadDenial;
use crate::identity::data::KindId;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::candidate_source::program::ManagedIndexSourceProgram;
use crate::validation::custom_rule::structural_views::StructuralRelationView;
use crate::validation::custom_rule::{CustomInvariantWorkMeter, CustomPreparationBudget};
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_contracts::internal_port::validation::candidate_source::program::ManagedIndexSourceProgramOwnerPort as _;

pub(in crate::validation::candidate_source) fn prepare<'a>(
    context: &'a SourcePreparationContext<'a>,
    budget: &CustomPreparationBudget,
    authored: Option<&MergedCommitPlan>,
    validation_view: impl FnOnce() -> InvariantRuntimeView<'a>,
) -> Result<
    (
        Option<Arc<PreparedSourceImpactHistory>>,
        Vec<DiscoveredSource<'a>>,
    ),
    Denial,
> {
    let SourcePreparationContext {
        state,
        outcome,
        allocations,
        interpretation,
        registry,
        home_target,
        execution,
        selected,
        proposal,
        records: _,
    } = *context;
    let catalog = &registry.source_history;
    let Some(plan) = authored else {
        return Ok((None, Vec::new()));
    };
    if !catalog.has_programs() || interpretation.materialization_mode().is_some() {
        return Ok((None, Vec::new()));
    }
    let before = InvariantStateView::new(&selected, selected.version_id());
    let overlay = OverlayStateView::new(&selected, state);
    let after = InvariantStateView::new(&overlay, proposal.proposed_version_id());
    let runtime_view = validation_view();
    let observation = InvariantObservation::applied_with_proposal(
        OverlayStateView::new(&selected, state),
        selected.version_id(),
        Some(proposal.clone()),
        &outcome.created_entities,
    );
    let mut discoveries = Vec::new();
    let Some(input) = AuthoredSourceInputs::collect(
        execution,
        budget,
        allocations,
        &outcome.effect,
        plan,
        &before,
        &after,
    )?
    else {
        return Ok((None, discoveries));
    };
    admission::work(
        budget,
        input.records.len().checked_mul(2).ok_or_else(|| {
            admission::storage(
                crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
            )
        })?,
    )?;
    let groups = if home_target {
        registry.all_source_groups().iter().collect()
    } else {
        registry.source_groups(input.entity_kinds(&before, &after), input.relation_kinds())
    };
    let adjacency = input.deltas(budget)?;
    let mut seen = ExecutionOrderedMap::<usize, ()>::default();
    let mut impacts = ExecutionOrderedMap::default();
    for group in groups {
        for program in group.programs() {
            admission::work(budget, 1)?;
            admission::work(budget, catalog.len())?;
            let slot = catalog.slot(program).map_err(|_| Denial::ForeignSource)?;
            admission::ordered(budget, seen.len(), size_of::<usize>())?;
            if seen.contains_key(&slot) {
                continue;
            }
            seen.insert_cloned_native_for_request(&slot, &(), execution)
                .map_err(admission::map)?;
            program
                .require_access(group.access_contract())
                .map_err(DerivedRelationReadDenial::from)?;
            let work = CustomInvariantWorkMeter::new(group.maximum_work_units()).with_lease(budget);
            let access = Arc::new(group.access_contract().clone());
            let old = StructuralRelationView::new(
                before.clone(),
                execution,
                work.clone(),
                access.clone(),
            )
            .with_structural_source(&runtime_view, &observation, true);
            let new = StructuralRelationView::new(after.clone(), execution, work.clone(), access)
                .with_structural_source(&runtime_view, &observation, false);
            let records = source_records(execution, budget, program, &input, &before, &after)?;
            let closure = if home_target {
                super::super::scope_closure::discover_home_target(
                    runtime_view.selected_registry.registry(),
                    program,
                    context.records,
                    &new,
                )?
            } else {
                super::super::scope_closure::discover(
                    runtime_view.selected_registry.registry(),
                    program,
                    &records,
                    &adjacency,
                    &old,
                    &new,
                )?
            };
            for (scope, rows) in &closure.scopes {
                let mut retained = ExecutionOrderedMap::default();
                for row in rows {
                    admission::ordered(budget, retained.len(), size_of_val(row))?;
                    retained
                        .insert_cloned_native_for_request(row, &(), execution)
                        .map_err(admission::map)?;
                }
                let key = ManagedSourceImpactKey {
                    program: slot,
                    scope: *scope,
                };
                admission::ordered(budget, impacts.len(), size_of_val(&key))?;
                impacts
                    .insert_cloned_native_for_request(
                        &key,
                        &ManagedSourceImpactRows(retained),
                        execution,
                    )
                    .map_err(admission::map)?;
            }
            // History uses its original closure before maintenance can add sibling invocation scopes.
            if let Some(discovery) = DiscoveredSource::capture(
                context, group, program, &records, &adjacency, closure, work, budget,
            )? {
                discoveries.try_reserve_exact(1).map_err(|_| {
                    admission::storage(
                        crate::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                    )
                })?;
                discoveries.push(discovery);
            }
        }
    }
    let basis = interpretation.physical_basis();
    let branch = basis.identity().branch_id();
    let memory = admission::header::<PreparedSourceImpactHistory>(execution, branch.0.len())?;
    admission::work(budget, branch.0.len())?;
    let result = Some(Arc::new(PreparedSourceImpactHistory {
        catalog: catalog.clone(),
        runtime: basis.identity().runtime_instance_id(),
        branch: branch.clone(),
        parent: basis.observation().commit_id(),
        proposed: proposal.proposed_version_id(),
        records: input.records,
        incidence: input.incidence,
        impacts,
        _memory: memory,
    }));
    Ok((result, discoveries))
}

fn source_records(
    execution: ExecutionRequest<'_, '_>,
    budget: &CustomPreparationBudget,
    program: &ManagedIndexSourceProgram,
    input: &AuthoredSourceInputs,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
) -> Result<Vec<RecordRef>, Denial> {
    let count = input.records.len();
    let bytes = count.checked_mul(size_of::<RecordRef>()).ok_or_else(|| {
        admission::storage(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
        )
    })?;
    if !budget.try_plan_item(bytes as u64) {
        return Err(DerivedRelationReadDenial::Definition(
            crate::derived_relation::definition::DerivedRelationDenial::WorkBudgetExceeded,
        )
        .into());
    }
    let mut records = Vec::new();
    records.try_reserve_exact(count).map_err(|_| {
        admission::storage(
            crate::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
        )
    })?;
    for (record, usage) in input.records.iter() {
        let RecordRef::Entity(id) = record else {
            continue;
        };
        admission::work(budget, 2)?;
        for metadata in [before.entity_metadata(*id), after.entity_metadata(*id)]
            .into_iter()
            .flatten()
        {
            if record_use_includes(*usage, program, metadata.kind_id, budget)? {
                records.push(record.clone());
                break;
            }
        }
    }
    execution
        .consult()
        .map_err(|cause| admission::storage(cause.into()))?;
    Ok(records)
}

fn record_use_includes(
    usage: super::AuthoredSourceRecordUse,
    program: &ManagedIndexSourceProgram,
    kind: KindId,
    budget: &CustomPreparationBudget,
) -> Result<bool, Denial> {
    admission::work(budget, 1)?;
    if usage.lifecycle {
        if kind == program.scope_kind() || kind == program.row_kind() {
            return Ok(true);
        }
        for &(_, from, to) in program.physical_sources() {
            admission::work(budget, 1)?;
            if kind == from || kind == to {
                return Ok(true);
            }
        }
    }
    if usage.fields {
        if kind == program.row_kind() && !program.fields().is_empty() {
            return Ok(true);
        }
        for route in program.projection_routes() {
            admission::work(budget, 1)?;
            if kind == route.endpoint_kind() && !route.fields().is_empty() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
