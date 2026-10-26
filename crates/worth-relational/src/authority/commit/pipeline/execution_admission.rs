use super::authority_context::AuthoritativeCommitContext;
use super::bulk_mutation_telemetry::record_bulk_mutation_telemetry;
use super::rejection::stale_strategy_validation_basis;
use crate::mvcc::RelationalTransactionValidationInput;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    CommitConflict, CommitLog, CommitPhaseTiming, ConflictClass, MergedCommitPlan,
    TransactionCommitError, TransactionId,
};

pub(super) struct AdmittedCommitExecution {
    transaction_id: TransactionId,
    validation_input: RelationalTransactionValidationInput,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
    phase_timing: CommitPhaseTiming,
    commit_log: CommitLog,
    authority_input: super::authority_context::CommitAuthorityInput,
    prepared_scope: Option<super::authority_context::PreparedAuthorityScope>,
    merge_execution_accounting: Option<super::authority_context::MergeExecutionAccounting>,
    merge_execution_diagnostics_plan: Option<crate::merge::data::MergeExecutionDiagnosticsPlan>,
    prevalidated_commit_boundary: Option<crate::validation::engine::InvariantExecutionResult>,
    prevalidated_mutation_sensitive: Option<crate::validation::engine::InvariantExecutionResult>,
    prevalidated_snapshot_publication: Option<crate::validation::engine::InvariantExecutionResult>,
    strategy_commit_artifacts: Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
    diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    complexity_before: crate::performance::data::RuntimeComplexityCounters,
    prior_complexity_delta: crate::performance::data::RuntimeComplexityCounters,
}

pub(super) struct AdmittedCommitPhaseView<'a> {
    transaction_id: TransactionId,
    validation_input: &'a RelationalTransactionValidationInput,
    authority_input: &'a super::authority_context::CommitAuthorityInput,
    commit_log: &'a mut CommitLog,
    phase_timing: &'a mut CommitPhaseTiming,
}

impl<'a> AdmittedCommitPhaseView<'a> {
    pub(super) fn into_parts(
        self,
    ) -> (
        TransactionId,
        &'a RelationalTransactionValidationInput,
        &'a MergedCommitPlan,
        Option<&'a crate::transactions::data::MergeCommitMutationPlan>,
        &'a mut CommitLog,
        &'a mut CommitPhaseTiming,
    ) {
        let (merged_plan, merge_history_plan) = match self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(plan) => (plan, None),
            super::authority_context::CommitAuthorityInput::Merge(plan) => {
                (&plan.merged_plan, Some(plan))
            }
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

impl AdmittedCommitExecution {
    pub(super) fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub(super) fn merged_plan(&self) -> &MergedCommitPlan {
        match &self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(plan) => plan,
            super::authority_context::CommitAuthorityInput::Merge(plan) => &plan.merged_plan,
        }
    }

    pub(super) fn selected_branch_state(&self) -> &crate::branch::SelectedRelationalBranchState {
        &self.selected_branch_state
    }

    pub(super) fn merge_history_plan(
        &self,
    ) -> Option<&crate::transactions::data::MergeCommitMutationPlan> {
        match &self.authority_input {
            super::authority_context::CommitAuthorityInput::Mutation(_) => None,
            super::authority_context::CommitAuthorityInput::Merge(plan) => Some(plan),
        }
    }

    pub(super) fn validation_input(&self) -> &RelationalTransactionValidationInput {
        &self.validation_input
    }

    pub(super) fn release_transaction_retention(&mut self) {
        self.validation_input.release_transaction_retention();
    }

    pub(super) fn take_prepared_scope(
        &mut self,
    ) -> Option<super::authority_context::PreparedAuthorityScope> {
        self.prepared_scope.take()
    }

    pub(super) fn accept_merge_draft(
        &mut self,
        draft: super::merge_draft::AcceptedMergeDraft,
    ) -> super::authority_context::PreparedAuthorityScope {
        self.prevalidated_commit_boundary = Some(draft.boundary);
        self.prevalidated_mutation_sensitive = Some(draft.mutation_sensitive);
        self.prevalidated_snapshot_publication = Some(draft.publication);
        draft.scope
    }

    pub(super) fn take_prevalidated_boundary(
        &mut self,
    ) -> Option<crate::validation::engine::InvariantExecutionResult> {
        self.prevalidated_commit_boundary.take()
    }

    pub(super) fn take_prevalidated_mutation_sensitive(
        &mut self,
    ) -> Option<crate::validation::engine::InvariantExecutionResult> {
        self.prevalidated_mutation_sensitive.take()
    }

    pub(super) fn take_prevalidated_snapshot_publication(
        &mut self,
    ) -> Option<crate::validation::engine::InvariantExecutionResult> {
        self.prevalidated_snapshot_publication.take()
    }

    pub(super) fn commit_phase_state(&mut self) -> (&mut CommitLog, &mut CommitPhaseTiming) {
        (&mut self.commit_log, &mut self.phase_timing)
    }

    pub(super) fn phase_view(&mut self) -> AdmittedCommitPhaseView<'_> {
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
    ) -> Option<&crate::commit_strategies::data::StrategyCommitArtifactBundle> {
        self.strategy_commit_artifacts.as_ref()
    }

    pub(super) fn take_merge_accounting(
        &mut self,
    ) -> Option<super::authority_context::MergeExecutionAccounting> {
        self.merge_execution_accounting.take()
    }

    pub(super) fn merge_execution_diagnostics_plan(
        &self,
    ) -> Option<&crate::merge::data::MergeExecutionDiagnosticsPlan> {
        self.merge_execution_diagnostics_plan.as_ref()
    }

    pub(super) fn into_result_parts(
        self,
    ) -> (
        TransactionId,
        CommitPhaseTiming,
        CommitLog,
        Option<crate::commit_strategies::data::StrategyCommitArtifactBundle>,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
        crate::performance::data::RuntimeComplexityCounters,
        crate::performance::data::RuntimeComplexityCounters,
    ) {
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
        diagnostics: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ) {
        self.diagnostics.extend(diagnostics);
    }
}

/// Admit the selected context after its caller's Admission stage consult.
pub(super) fn admit_commit_execution(
    runtime: &RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<AdmittedCommitExecution, TransactionCommitError> {
    let complexity_before = context
        .complexity_baseline
        .clone()
        .unwrap_or_else(|| runtime.performance_access().complexity_counters_snapshot());
    enforce_validated_strategy_basis(
        runtime,
        context.transaction_id,
        &context.validation_input,
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
            runtime
                .selected_branch_state(context.validation_input.basis())
                .map_err(TransactionCommitError::preparation)
        })?;
    if let Some(telemetry) = context.bulk_mutation_telemetry.as_ref() {
        record_bulk_mutation_telemetry(runtime, telemetry);
    }
    if matches!(
        context.validation_input.basis().reference().target(),
        worth_foundational::FoundationalBranchTarget::Basis(_)
    ) {
        runtime
            .history
            .record_retained_history_head_lookup(context.validation_input.target_branch());
    }
    Ok(AdmittedCommitExecution {
        transaction_id: context.transaction_id,
        validation_input: context.validation_input,
        selected_branch_state,
        phase_timing: context.phase_timing,
        commit_log: CommitLog::new(),
        authority_input: context.authority_input,
        prepared_scope: context.prepared_scope,
        merge_execution_accounting: context.merge_execution_accounting,
        merge_execution_diagnostics_plan: context.merge_execution_diagnostics_plan,
        prevalidated_commit_boundary: context.prevalidated_commit_boundary,
        prevalidated_mutation_sensitive: context.prevalidated_mutation_sensitive,
        prevalidated_snapshot_publication: context.prevalidated_snapshot_publication,
        strategy_commit_artifacts: context.strategy_commit_artifacts,
        diagnostics: Vec::new(),
        complexity_before,
        prior_complexity_delta: context.prior_complexity_delta,
    })
}

fn enforce_validated_strategy_basis(
    runtime: &RelationalPreparationRuntime,
    transaction_id: TransactionId,
    validation_input: &crate::mvcc::RelationalTransactionValidationInput,
    validated_against_commit_id: Option<crate::history::data::CommitId>,
    validated_against_version_id: Option<crate::identity::data::VersionId>,
    validated_against_branch_version: Option<crate::branch::RelationalBranchVersion>,
    proposal_identity: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
) -> Result<(), TransactionCommitError> {
    let branch = validation_input.target_branch().clone();
    if validation_input.basis().identity().runtime_instance_id() != runtime.runtime_instance_id() {
        return Err(TransactionCommitError::conflict(CommitConflict::new(
            ConflictClass::ForeignRuntime {
                expected_runtime_instance_id: runtime.runtime_instance_id(),
                actual_runtime_instance_id: validation_input
                    .basis()
                    .identity()
                    .runtime_instance_id(),
            },
        )));
    }
    if let Some(identity) = proposal_identity
        && (identity.runtime_instance_id() != runtime.runtime_instance_id()
            || identity.transaction_id() != transaction_id
            || identity.branch_observation() != validation_input.basis().reference()
            || identity.branch_version() != validation_input.basis().truth_version())
    {
        return Err(stale_strategy_validation_basis(
            "proposal identity does not match the owner-issued transaction basis",
        ));
    }
    if !validation_input.basis().is_current() {
        return Err(stale_strategy_validation_basis(
            "transaction branch binding is no longer current",
        ));
    }
    if let Some(validated_branch_version) = validated_against_branch_version {
        let binding = validation_input.basis();
        let Some(cell) = runtime.history.branch_cell(&branch) else {
            return Err(stale_strategy_validation_basis(
                "validated transaction branch is no longer registered",
            ));
        };
        if cell.identity() != binding.identity()
            || cell.observation() != *binding.reference()
            || cell.truth_version() != validated_branch_version
        {
            return Err(stale_strategy_validation_basis(
                "validated transaction no longer matches the current branch reference",
            ));
        }
    }
    let current_commit = validation_input.basis().commit_identity();
    let current_version = runtime.admitted_branch_basis_version(validation_input.basis());
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
    current_version: Option<crate::identity::data::VersionId>,
    validated_version: Option<crate::identity::data::VersionId>,
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
