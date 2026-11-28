//! Match the active configuration to this preparation operation's counters.
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::prepare::working_state_inputs::WorkingStatePreparation;

impl<'a> From<&'a RelationalPreparationRuntime> for WorkingStatePreparation<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            adjacency_policy: &runtime.config.storage.adjacency_policy,
            instrumentation: &runtime.services.instrumentation,
        }
    }
}

use crate::authority::commit::phases::prepare::working_state_inputs::{
    WorkingStateDraft, WorkingStateDraftRuntime, WorkingStateScope,
};
use crate::mvcc::transaction::staged_plan_normalization::StagedPlanNormalization;
use crate::mvcc::BranchBoundRelationalTransaction;

impl<'a> From<&'a RelationalPreparationRuntime> for WorkingStateDraftRuntime<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            working_state: runtime.into(),
            normalization: runtime.staged_plan_normalization(),
            cross_context: runtime.config.storage.cross_context_policy,
            commit_work: runtime.commit_work_budget.as_ref(),
        }
    }
}
impl<'a> From<&'a mut BranchBoundRelationalTransaction> for WorkingStateDraft<'a> {
    fn from(transaction: &'a mut BranchBoundRelationalTransaction) -> Self {
        let mutation_schema = BranchBoundRelationalTransaction::select_mutation_schema(
            &transaction.home_target,
            &transaction.schema_authority,
        );
        Self {
            transaction: transaction.transaction_id,
            overlay: &mut transaction.overlay,
            footprint: &mut transaction.footprint,
            basis_schema: transaction.schema_authority.registry(),
            mutation_schema,
            merge_parent_count: 0,
            coupled_commands: transaction
                .authored_coupling
                .as_ref()
                .map(|_| &transaction.prepared_coupled_commands),
        }
    }
}
impl<'a> From<&'a BranchBoundRelationalTransaction> for WorkingStateScope<'a> {
    fn from(transaction: &'a BranchBoundRelationalTransaction) -> Self {
        Self {
            footprint: &transaction.footprint,
            schema: transaction.mutation_schema_authority(),
            merge_parent_count: 0,
        }
    }
}

impl RelationalPreparationRuntime {
    pub(crate) fn staged_plan_normalization(&self) -> StagedPlanNormalization<'_> {
        StagedPlanNormalization {
            policy: self.config.identity.client_key_symbol_policy,
            symbols: &self.services.symbols,
        }
    }
}
