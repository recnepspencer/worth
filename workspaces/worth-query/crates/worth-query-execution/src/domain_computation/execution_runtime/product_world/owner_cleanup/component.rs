use worth_proof::TransitionOutcome;
use worth_relational::facade::branch::{
    RelationalBranchDeletionOutcome, RelationalBranchTransitionStop,
};
use worth_runtime_world::facade::OwnerRetirementWork;
use worth_signal::facade::branch::{SignalBranchRetirementReason, SignalOwnerCancellationSource};

use super::{OwnerCleanupRetirements, WorthQueryProductBranchOwnerCleanupDenial};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryProductRuntime {
    pub(super) fn retire_owner_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work: &OwnerRetirementWork,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        match work {
            OwnerRetirementWork::RelationalBranchRetirement { identity, .. } => {
                let prepared = retirements.prepare(execution, identity)?;
                match self.relational_lifecycle.delete_branch(execution, identity) {
                    Ok(RelationalBranchDeletionOutcome::Deleted(deleted)) => {
                        retirements.deleted(prepared, deleted);
                        Ok(())
                    }
                    Ok(RelationalBranchDeletionOutcome::WaitingForActiveOperations(_)) => Err(
                        WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive,
                    ),
                    Err(RelationalBranchTransitionStop::DurabilityUnresolved(_)) => Err(
                        WorthQueryProductBranchOwnerCleanupDenial::RelationalDurabilityUnresolved,
                    ),
                    Err(_) => Err(WorthQueryProductBranchOwnerCleanupDenial::RelationalOwnerDenied),
                }
            }
            OwnerRetirementWork::SignalBranchRetirement { reference, .. } => {
                let lifecycle=match &self.signal_lifecycle {
                    super::super::signal_lifecycle::SignalLifecycleServices::Memory(port)=>port,
                    super::super::signal_lifecycle::SignalLifecycleServices::AuthoredHome=>return Err(WorthQueryProductBranchOwnerCleanupDenial::DurableSignalRetirementRequired),
                };

                let basis = self
                    .signal_basis
                    .observe_current(reference)
                    .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied)?;
                let plan = match lifecycle.plan_retirement_exact(
                    basis,
                    SignalBranchRetirementReason::DependencyCancellation,
                ) {
                    TransitionOutcome::Success(plan) => plan,
                    TransitionOutcome::Denied(_) => {
                        return Err(WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied);
                    }
                };
                let cancellation = SignalOwnerCancellationSource::new();
                match lifecycle.retire_exact(plan, &cancellation.token()) {
                    TransitionOutcome::Success(_) => Ok(()),
                    TransitionOutcome::Denied(_) => {
                        Err(WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied)
                    }
                }
            }
        }
    }
}
