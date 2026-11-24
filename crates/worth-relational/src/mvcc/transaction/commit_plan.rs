use super::staged_plan::{MergedPlanPreparationTiming, StagedPlanValidation};
use crate::authority::intent_merge::validate_branch_locality;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{CommitConflict, ConflictClass};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

impl crate::mvcc::BranchBoundRelationalTransaction {
    pub(crate) fn validate_staged_branch_locality(
        &self,
        current_state: &impl CurrentPartitionAccess,
        interner: &crate::symbols::data::StringInterner,
    ) -> Result<(), CommitConflict> {
        validate_branch_locality(current_state, self.batches(), interner)
    }

    pub(crate) fn build_merged_plan_for_state(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        current_state: &impl CurrentPartitionAccess,
        intents: super::OwnedNormalizedPlan,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::OwnedNormalizedPlan, crate::transactions::data::TransactionCommitError> {
        self.build_merged_plan_for_state_with_timing(runtime, current_state, intents, request)
            .map(|(plan, _)| plan)
    }

    pub(crate) fn build_merged_plan_for_state_with_timing(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        current_state: &impl CurrentPartitionAccess,
        intents: super::OwnedNormalizedPlan,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (super::OwnedNormalizedPlan, MergedPlanPreparationTiming),
        crate::transactions::data::TransactionCommitError,
    > {
        StagedPlanValidation {
            schema: self.mutation_schema_authority().registry(),
            cross_context: runtime.config.storage.cross_context_policy,
            instrumentation: &runtime.services.instrumentation,
            coupled_commands: self
                .authored_coupling
                .as_ref()
                .map(|_| &self.prepared_coupled_commands),
        }
        .validate(current_state, intents, request)
    }

    pub(crate) fn normalized_intents_for_merge(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::OwnedNormalizedPlan, crate::transactions::data::TransactionCommitError> {
        runtime.staged_plan_normalization().normalized(
            self.transaction_id,
            &mut self.overlay,
            &mut self.footprint,
            request,
        )
    }

    pub(crate) fn ensure_runtime_affinity(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_instance(runtime.runtime_instance_id())
    }

    pub(crate) fn ensure_runtime_affinity_for_runtime(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_instance(runtime.runtime_instance_id())
    }

    fn ensure_runtime_affinity_for_instance(
        &self,
        runtime_instance_id: u64,
    ) -> Result<(), CommitConflict> {
        if self.basis.identity().runtime_instance_id() == runtime_instance_id {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::ForeignRuntime {
                expected_runtime_instance_id: runtime_instance_id,
                actual_runtime_instance_id: self.basis.identity().runtime_instance_id(),
            }))
        }
    }

    pub(crate) fn ensure_current_basis(
        &self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity(runtime)?;
        if self.basis.is_current() {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::StaleValidationBasis {
                detail: "transaction basis is no longer current".to_owned(),
            }))
        }
    }

    pub(crate) fn ensure_current_basis_for_runtime(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), CommitConflict> {
        self.ensure_runtime_affinity_for_runtime(runtime)?;
        if self.basis.is_current() {
            Ok(())
        } else {
            Err(CommitConflict::new(ConflictClass::StaleValidationBasis {
                detail: "transaction basis is no longer current".to_owned(),
            }))
        }
    }
}
