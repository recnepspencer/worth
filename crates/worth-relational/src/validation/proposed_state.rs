//! Evaluate a proposed state against the admitted invariant view.
use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::branch::selected_read::SelectedRelationalBranchRead;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::data::contracts::InvariantPlanContract;
use crate::validation::data::groups::InvariantGroupSet;
use crate::validation::engine::{
    InvariantEngine, InvariantExecutionDisposition, InvariantExecutionMetadata,
    InvariantExecutionRequest, InvariantExecutionResult, InvariantObservation,
    InvariantRequestProfile, InvariantRuntimeView,
};

pub(crate) fn begin_proposed_state(
    view: &InvariantRuntimeView<'_>,
    selected: &SelectedRelationalBranchRead<'_>,
    working: &WorkingState,
    version: crate::identity::data::VersionId,
    plan: RelationalSemanticPlan<'_>,
    proposal: Option<
        &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    >,
    aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
    prepare_sources: &mut dyn FnMut() -> Result<(), TransactionCommitError>,
) -> Result<crate::validation::engine::StartedProposedPass, TransactionCommitError> {
    let aggregate = aggregate.map(|source| source.invocation(plan.commands()));
    let observation = InvariantObservation::committed_branch_with_proposed(
        selected,
        working,
        version,
        proposal.cloned(),
    )
    .with_aggregate(aggregate);
    let request = InvariantExecutionRequest::from_semantic_plan(
        execution,
        InvariantRequestProfile::CommitBoundary,
        view,
        observation,
        selected.version_id(),
        selected.version_id(),
        Some(plan),
        Some(plan.invariant_contract()),
        control,
    )?;
    let started =
        InvariantEngine::from_view(view).begin_proposed(execution, request, prepare_sources);
    match aggregate {
        Some(source) => source.settle(started, execution, control),
        None => started,
    }
}

pub(crate) fn complete_proposed_state(
    view: &InvariantRuntimeView<'_>,
    selected: &SelectedRelationalBranchRead<'_>,
    working: &WorkingState,
    pending: crate::validation::engine::PendingProposedPass,
    created_entities: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    version: crate::identity::data::VersionId,
    plan: RelationalSemanticPlan<'_>,
    proposal: Option<
        &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    >,
    aggregate: Option<crate::aggregate_identity::AggregateProposalBorrow<'_>>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<InvariantExecutionResult, TransactionCommitError> {
    let profile = InvariantRequestProfile::ProposedState;
    let observation = InvariantObservation::committed_branch_with_proposed(
        selected,
        working,
        version,
        proposal.cloned(),
    )
    .with_applied_created_entities(created_entities);
    // Aggregate logical changes were captured from the actual applied
    // effect. Its command constituent is used only for native consequences.
    let aggregate = aggregate.map(|source| source.invocation(plan.commands()));
    let observation = observation.with_aggregate(aggregate);
    let observation_kind = observation.kind();
    let plan_contract = (!view.rejudges_home_target()).then(|| plan.invariant_contract());
    let consumed_groups = profile.consumed_groups();
    if plan_contract.is_some_and(|contract| !contract.intersects_consumed_groups(consumed_groups)) {
        return Ok(InvariantExecutionResult::skipped(metadata(
            profile,
            observation_kind,
            version,
            plan_contract,
            InvariantGroupSet::empty(),
            crate::validation::data::groups::InvariantCostClass::Global,
            InvariantExecutionDisposition::SkippedByPlanContract,
            proposal,
        )));
    }
    let request = InvariantExecutionRequest::from_semantic_plan(
        execution,
        profile,
        view,
        observation,
        version,
        selected.version_id(),
        Some(plan),
        plan_contract,
        control,
    )?;
    if !request.should_execute_anything() {
        return Ok(InvariantExecutionResult::skipped(metadata(
            profile,
            observation_kind,
            version,
            plan_contract,
            request.applicable_groups(),
            request.max_cost(),
            InvariantExecutionDisposition::SkippedByMayBreakMask,
            request.proposal_identity(),
        )));
    }
    let result = InvariantEngine::from_view(view).complete_proposed(execution, request, pending);
    match aggregate {
        Some(source) => source.settle(result, execution, control),
        None => result,
    }
}

fn metadata(
    profile: InvariantRequestProfile,
    observation: crate::validation::engine::InvariantObservationKind,
    version: crate::identity::data::VersionId,
    contract: Option<InvariantPlanContract>,
    groups: InvariantGroupSet,
    cost: crate::validation::data::groups::InvariantCostClass,
    disposition: InvariantExecutionDisposition,
    proposal: Option<
        &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    >,
) -> InvariantExecutionMetadata {
    InvariantExecutionMetadata::new(
        profile.execution_point(),
        observation,
        version,
        version,
        profile.consumed_groups(),
        groups,
        cost,
        disposition,
        contract,
        true,
        None,
        Vec::new(),
        None,
        proposal.cloned(),
    )
}

/// Find the first enforced verdict of the requested effect in reduction order.
pub(crate) fn enforced_failure(
    result: &InvariantExecutionResult,
    effect: crate::validation::data::execution::InvariantFailureEffect,
) -> Option<crate::validation::engine::InvariantFailure> {
    result.results().iter().find_map(|row| {
        if row.failure_effect != effect || !row.blocks_proposed_state() {
            return None;
        }
        let crate::validation::data::execution::InvariantVerdict::Violation(violation) =
            &row.verdict
        else {
            return None;
        };
        Some(crate::validation::engine::InvariantFailure::new(
            row.execution_point,
            effect,
            violation.clone(),
        ))
    })
}
