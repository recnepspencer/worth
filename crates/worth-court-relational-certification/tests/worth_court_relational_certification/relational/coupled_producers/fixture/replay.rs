//! Cold WAL replay of a world's committed history. Recovery replays every
//! canonical commit through the ordinary mutation pipeline and requires each
//! replayed envelope to equal the durable one (`validate_recovered_history_parity`).
use super::*;
use worth_relational::facade::durability::{RecoveryVerificationMode, RelationalRecoveryStop};

impl World {
    /// Prepare an edit envelope whose authored prefix also applies `prefix`.
    pub fn prepare_with_prefix(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prefix: WorkerIntentBatch,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let owners = self.runtime.readmit_edit_coupled_owners(
            execution,
            &self.installed,
            &self.basis(),
            self.source,
        )?;
        self.prepare_with_owners(execution, owners, Some(prefix), assembly)
    }

    /// Prepare over admitted `owners` with an authored prefix that also applies `prefix`.
    pub fn prepare_owners_with_prefix(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        owners: RelationalAuthoredCoupledOwners,
        prefix: WorkerIntentBatch,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.prepare_with_owners(execution, owners, Some(prefix), assembly)
    }

    /// A fresh runtime over this world's schema, rebuilt by replaying its
    /// whole committed history from the canonical envelopes (no checkpoint).
    pub fn replay(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalRuntime, RelationalRecoveryStop> {
        let plan = self.runtime.durability().recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )?;
        let mut cold = RelationalRuntimeApi::builder()
            .schema_registry(
                self.registry
                    .clone()
                    .expect("a world built over a known schema"),
            )
            .build()
            .unwrap();
        cold.prepare_initial_schema_installation()
            .unwrap()
            .install(RelationalSchemaRegistry::new())
            .unwrap();
        cold.durability_recovery().recover(execution, plan)?;
        Ok(cold)
    }
}
