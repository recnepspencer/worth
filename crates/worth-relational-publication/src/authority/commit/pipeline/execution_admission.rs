use super::authority_context::AuthoritativeCommitContext;
use super::bulk_mutation_telemetry::record_bulk_mutation_telemetry;
use super::publication_validation::{CommitPublicationValidationInput, DraftValidationInput};
use super::rejection::stale_strategy_validation_basis;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::TransactionId;
use mutation::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

/// Admission owns the exact draft only until the consuming Draft handoff.
pub(super) struct AdmittedCommitExecution<V> {
    prepared_scope: Option<super::authority_context::PreparedAuthorityScope>,
    state: AdmittedCommitState<V>,
}
impl<V> AdmittedCommitExecution<V> {
    pub(super) fn into_parts(
        self,
    ) -> (
        AdmittedCommitState<V>,
        Option<super::authority_context::PreparedAuthorityScope>,
    ) {
        (self.state, self.prepared_scope)
    }
}

/// Metadata that remains live after the exact draft leaves admission.
pub struct AdmittedCommitState<V = CommitPublicationValidationInput> {
    transaction_id: TransactionId,
    validation_input: V,
    selected_branch_state: worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    phase_timing: CommitPhaseTiming,
    commit_log: CommitLog,
    authority_input: super::authority_context::CommitAuthorityInput,
    merge_execution_accounting: Option<super::authority_context::MergeExecutionAccounting>,
    merge_execution_diagnostics_plan: Option<contracts::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan>,
    prevalidated_invariants: Option<mutation::validation::engine::result::InvariantExecutionResult>,
    strategy_commit_artifacts: Option<contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle>,
    diagnostics: Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    complexity_before:
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    prior_complexity_delta:
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
}

pub(super) struct AdmittedCommitPhaseView<'a, V> {
    transaction_id: TransactionId,
    validation_input: &'a V,
    authority_input: &'a super::authority_context::CommitAuthorityInput,
    commit_log: &'a mut CommitLog,
    phase_timing: &'a mut CommitPhaseTiming,
}

impl<'a, V> AdmittedCommitPhaseView<'a, V> {
    pub(super) fn into_parts(
        self,
    ) -> (
        TransactionId,
        &'a V,
        RelationalSemanticPlan<'a>,
        Option<
            &'a contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan,
        >,
        &'a mut CommitLog,
        &'a mut CommitPhaseTiming,
    ) {
        let (merged_plan, merge_history_plan) = match self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(plan) => (plan.view(), None),
            super::authority_context::CommitAuthorityInput::Merge(plan) => (
                RelationalSemanticPlan::Mutation(&plan.plan().merged_plan),
                Some(plan.plan()),
            ),
        };
        (
            self.transaction_id,
            self.validation_input,
            merged_plan,
            merge_history_plan,
            self.commit_log,
            self.phase_timing,
        )
    }
}

impl<V> AdmittedCommitState<V> {
    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    /// Authored/consequence commands only. Complete scope uses semantic_plan.
    pub fn command_plan(&self) -> &MergedCommitPlan {
        match &self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(plan) => {
                plan.view().commands()
            }
            super::authority_context::CommitAuthorityInput::Merge(plan) => &plan.plan().merged_plan,
        }
    }

    pub fn semantic_plan(&self) -> RelationalSemanticPlan<'_> {
        match &self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(plan) => plan.view(),
            super::authority_context::CommitAuthorityInput::Merge(plan) => {
                RelationalSemanticPlan::Mutation(&plan.plan().merged_plan)
            }
        }
    }

    pub fn selected_branch_state(
        &self,
    ) -> &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState{
        &self.selected_branch_state
    }

    pub fn merge_history_plan(
        &self,
    ) -> Option<&contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan>
    {
        match &self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(_) => None,
            super::authority_context::CommitAuthorityInput::Merge(plan) => Some(plan.plan()),
        }
    }

    pub fn validation_input(&self) -> &V {
        &self.validation_input
    }

    pub(super) fn accept_merge_draft(
        &mut self,
        draft: super::merge_draft::AcceptedMergeDraft,
    ) -> super::authority_context::PreparedAuthorityScope {
        self.prevalidated_invariants = Some(draft.invariants);
        draft.scope
    }

    pub(super) fn take_prevalidated_invariants(
        &mut self,
    ) -> Option<mutation::validation::engine::result::InvariantExecutionResult> {
        self.prevalidated_invariants.take()
    }

    pub(super) fn commit_phase_state(&mut self) -> (&mut CommitLog, &mut CommitPhaseTiming) {
        (&mut self.commit_log, &mut self.phase_timing)
    }

    pub(super) fn phase_view(&mut self) -> AdmittedCommitPhaseView<'_, V> {
        AdmittedCommitPhaseView {
            transaction_id: self.transaction_id,
            validation_input: &self.validation_input,
            authority_input: &self.authority_input,
            commit_log: &mut self.commit_log,
            phase_timing: &mut self.phase_timing,
        }
    }

    pub(super) fn phase_timing_mut(&mut self) -> &mut CommitPhaseTiming {
        &mut self.phase_timing
    }

    pub(super) fn strategy_artifacts(
        &self,
    ) -> Option<&contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle>{
        self.strategy_commit_artifacts.as_ref()
    }

    pub(super) fn take_merge_accounting(
        &mut self,
    ) -> Option<super::authority_context::MergeExecutionAccounting> {
        self.merge_execution_accounting.take()
    }

    pub(super) fn merge_execution_diagnostics_plan(
        &self,
    ) -> Option<&contracts::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan> {
        self.merge_execution_diagnostics_plan.as_ref()
    }

    pub(super) fn into_result_parts(
        self,
    ) -> (
        TransactionId,
        CommitPhaseTiming,
        CommitLog,
        Option<contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle>,
        Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    ){
        (
            self.transaction_id,
            self.phase_timing,
            self.commit_log,
            self.strategy_commit_artifacts,
            self.diagnostics,
            self.complexity_before,
            self.prior_complexity_delta,
        )
    }

    pub(super) fn append_diagnostics(
        &mut self,
        diagnostics: Vec<contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact>,
    ) {
        self.diagnostics.extend(diagnostics);
    }
}

impl AdmittedCommitState {
    pub(super) fn release_transaction_retention(&mut self) {
        self.validation_input.retention = None;
    }
}

pub mod publication_handoff;

/// The admitted commit reads the same branch cells and records in its own round.
pub struct CommitExecutionAdmission<'a> {
    pub history: &'a RelationalPreparationHistory,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub runtime_identity: u64,
}

/// Admit the selected context after its caller's Admission stage consult.
pub(super) fn admit_commit_execution<V: DraftValidationInput>(
    inputs: CommitExecutionAdmission<'_>,
    context: AuthoritativeCommitContext<V>,
) -> Result<AdmittedCommitExecution<V>, TransactionCommitError> {
    let complexity_before = context
        .complexity_baseline
        .clone()
        .unwrap_or_else(|| inputs.instrumentation.complexity_snapshot());
    enforce_validated_strategy_basis(
        &inputs,
        context.transaction_id,
        context.validation_input.basis(),
        context.validated_against_commit_id,
        context.validated_against_version_id,
        context.validated_against_branch_version,
        context
            .prepared_scope
            .as_ref()
            .map(|scope| &scope.mutation_proposal.0),
    )?;
    let selected_branch_state = context
        .prepared_scope
        .as_ref()
        .map(|scope| scope.selected_branch_state.clone())
        .map(Ok)
        .unwrap_or_else(|| {
            worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState::from_owner_basis(
                inputs.runtime_identity,
                context.validation_input.basis(),
            )
            .map_err(TransactionCommitError::preparation)
        })?;
    if let Some(telemetry) = context.bulk_mutation_telemetry.as_ref() {
        record_bulk_mutation_telemetry(inputs.instrumentation, telemetry);
    }
    if matches!(
        context.validation_input.basis().reference().target(),
        worth_foundational::FoundationalBranchTarget::Basis(_)
    ) {
        inputs
            .history
            .record_retained_history_head_lookup(context.validation_input.target_branch());
    }
    Ok(AdmittedCommitExecution {
        prepared_scope: context.prepared_scope,
        state: AdmittedCommitState {
            transaction_id: context.transaction_id,
            validation_input: context.validation_input,
            selected_branch_state,
            phase_timing: context.phase_timing,
            commit_log: CommitLog::new(),
            authority_input: context.authority_input,
            merge_execution_accounting: context.merge_execution_accounting,
            merge_execution_diagnostics_plan: context.merge_execution_diagnostics_plan,
            prevalidated_invariants: context.prevalidated_invariants,
            strategy_commit_artifacts: context.strategy_commit_artifacts,
            diagnostics: Vec::new(),
            complexity_before,
            prior_complexity_delta: context.prior_complexity_delta,
        },
    })
}

fn enforce_validated_strategy_basis(
    inputs: &CommitExecutionAdmission<'_>,
    transaction_id: TransactionId,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    validated_against_commit_id: Option<contracts::history::data::coordinates::CommitId>,
    validated_against_version_id: Option<contracts::identity::data::primitives::VersionId>,
    validated_against_branch_version: Option<contracts::branch::version::RelationalBranchVersion>,
    proposal_identity: Option<
        &mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    >,
) -> Result<(), TransactionCommitError> {
    let branch = basis.identity().branch_id().clone();
    if basis.identity().runtime_instance_id() != inputs.runtime_identity {
        return Err(TransactionCommitError::conflict(CommitConflict::new(
            ConflictClass::ForeignRuntime {
                expected_runtime_instance_id: inputs.runtime_identity,
                actual_runtime_instance_id: basis.identity().runtime_instance_id(),
            },
        )));
    }
    if let Some(identity) = proposal_identity
        && (identity.runtime_instance_id() != inputs.runtime_identity
            || identity.transaction_id() != transaction_id
            || identity.branch_observation() != basis.reference()
            || identity.branch_version() != basis.truth_version())
    {
        return Err(stale_strategy_validation_basis(
            "proposal identity does not match the owner-issued transaction basis",
        ));
    }
    if !basis.is_current() {
        return Err(stale_strategy_validation_basis(
            "transaction branch binding is no longer current",
        ));
    }
    if let Some(validated_branch_version) = validated_against_branch_version {
        let Some(cell) = inputs.history.branch_cell(&branch) else {
            return Err(stale_strategy_validation_basis(
                "validated transaction branch is no longer registered",
            ));
        };
        if cell.identity() != basis.identity()
            || cell.observation() != *basis.reference()
            || cell.truth_version() != validated_branch_version
        {
            return Err(stale_strategy_validation_basis(
                "validated transaction no longer matches the current branch reference",
            ));
        }
    }
    let current_commit = basis.commit_identity();
    let current_version = inputs.history.admitted_branch_basis_version(basis);
    validate_version_basis(current_version, validated_against_version_id)?;
    if validated_against_commit_id.is_some()
        && current_commit.as_ref().map(|commit| commit.commit_id()) != validated_against_commit_id
    {
        return Err(stale_strategy_validation_basis(
            "validated strategy plan no longer matches the current committed commit basis",
        ));
    }
    Ok(())
}

fn validate_version_basis(
    current_version: Option<contracts::identity::data::primitives::VersionId>,
    validated_version: Option<contracts::identity::data::primitives::VersionId>,
) -> Result<(), TransactionCommitError> {
    let Some(validated_version) = validated_version else {
        return Ok(());
    };
    let Some(observed) = current_version else {
        return Err(stale_strategy_validation_basis(
            "owner-issued branch binding has no exact local version basis",
        ));
    };
    (validated_version == observed)
        .then_some(())
        .ok_or_else(|| {
            stale_strategy_validation_basis(
                "validated strategy plan no longer matches the current committed version basis",
            )
        })
}
