//! Complete conservative scope candidates from actual authored and physical inputs.
use super::{
    admission, collection::AuthoredSourceInputs, ManagedSourceImpactKey, ManagedSourceImpactRows,
    PreparedSourceImpactHistory,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::branch::SelectedRelationalBranchRead;
use crate::derived_relation::DerivedRelationReadDenial;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::{MergedCommitPlan, RecordRef};
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use crate::validation::custom_rule::{CustomInvariantWorkMeter, CustomPreparationBudget};
use crate::validation::data::StructuralRelationView;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(in crate::validation::candidate_source) fn prepare(
    draft: &AppliedMutationDraft,
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected: SelectedRelationalBranchRead<'_>,
    proposal: &crate::mvcc::RelationalMutationProposalIdentity,
    authored: Option<&MergedCommitPlan>,
) -> Result<Option<Arc<PreparedSourceImpactHistory>>, Denial> {
    let catalog = &runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .source_history;
    let Some(plan) = authored else {
        return Ok(None);
    };
    if !catalog.has_programs() || draft.source_interpretation.materialization_mode().is_some() {
        return Ok(None);
    }
    let before = InvariantStateView::new(&selected, selected.version_id());
    let overlay = OverlayStateView::new(&selected, draft.state());
    let after = InvariantStateView::new(&overlay, proposal.proposed_version_id());
    let runtime_view =
        InvariantRuntimeView::from_preparation_for_state(runtime, selected.selected().state());
    let observation = InvariantObservation::applied_with_proposal(
        OverlayStateView::new(&selected, draft.state()),
        selected.version_id(),
        Some(proposal.clone()),
        &draft.outcome.created_entities,
    );
    let mut result = None;
    super::super::applied::stage::prepare(execution, |budget| {
        let Some(input) =
            AuthoredSourceInputs::collect(execution, budget, draft, plan, &before, &after)?
        else {
            return Ok(());
        };
        admission::work(
            budget,
            input.records.len().checked_mul(2).ok_or_else(|| {
                admission::storage(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?,
        )?;
        let registrations = runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .source_registrations(input.entity_kinds(&before, &after), input.relation_kinds());
        let adjacency = input.deltas(budget)?;
        let mut seen = ExecutionOrderedMap::<usize, ()>::default();
        let mut impacts = ExecutionOrderedMap::default();
        for registration in registrations {
            for program in registration.managed_sources.iter() {
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
                    .require_access(registration.access_contract())
                    .map_err(DerivedRelationReadDenial::from)?;
                let work = CustomInvariantWorkMeter::new(registration.maximum_work_units())
                    .with_lease(budget);
                let access = Arc::new(registration.access_contract().clone());
                let old = StructuralRelationView::new(
                    before.clone(),
                    execution,
                    work.clone(),
                    access.clone(),
                )
                .with_structural_source(&runtime_view, &observation, true);
                let new =
                    StructuralRelationView::new(after.clone(), execution, work.clone(), access)
                        .with_structural_source(&runtime_view, &observation, false);
                let records = source_records(execution, budget, program, &input, &before, &after)?;
                let closure = super::super::scope_closure::discover(
                    runtime_view.selected_registry.registry(),
                    program,
                    &records,
                    &adjacency,
                    &old,
                    &new,
                )?;
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
            }
        }
        let basis = draft.source_interpretation.physical_basis();
        let branch = basis.identity().branch_id();
        let memory = admission::header::<PreparedSourceImpactHistory>(execution, branch.0.len())?;
        admission::work(budget, branch.0.len())?;
        result = Some(Arc::new(PreparedSourceImpactHistory {
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
        Ok(())
    })?;
    Ok(result)
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
        admission::storage(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
    })?;
    if !budget.try_plan_item(bytes as u64) {
        return Err(DerivedRelationReadDenial::Definition(
            crate::derived_relation::DerivedRelationDenial::WorkBudgetExceeded,
        )
        .into());
    }
    let mut records = Vec::new();
    records.try_reserve_exact(count).map_err(|_| {
        admission::storage(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
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
            if usage.includes(program, metadata.kind_id, budget)? {
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
