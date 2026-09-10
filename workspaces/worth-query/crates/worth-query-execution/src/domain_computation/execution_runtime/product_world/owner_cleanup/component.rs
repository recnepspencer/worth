use worth_proof::TransitionOutcome;
use worth_relational::facade::branch::RelationalBranchDeletionOutcome;
use worth_runtime_world::facade::OwnerRetirementWork;
use worth_signal::facade::branch::{SignalBranchRetirementReason, SignalOwnerCancellationSource};

use super::WorthQueryProductBranchOwnerCleanupDenial;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;

impl WorthQueryProductRuntime {
    pub(super) fn retire_owner_component(
        &self,
        work: &OwnerRetirementWork,
    ) -> Result<(), WorthQueryProductBranchOwnerCleanupDenial> {
        match work {
            OwnerRetirementWork::RelationalBranchRetirement { identity, .. } => {
                match self.relational_lifecycle.delete_branch(identity) {
                    Ok(RelationalBranchDeletionOutcome::Deleted(_)) => Ok(()),
                    Ok(RelationalBranchDeletionOutcome::WaitingForActiveOperations(_)) => Err(
                        WorthQueryProductBranchOwnerCleanupDenial::RelationalOperationsStillActive,
                    ),
                    Err(_) => Err(WorthQueryProductBranchOwnerCleanupDenial::RelationalOwnerDenied),
                }
            }
            OwnerRetirementWork::SignalBranchRetirement { reference, .. } => {
                let basis = self
                    .signal_basis
                    .observe_current(reference)
                    .map_err(|_| WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied)?;
                let plan = match self.signal_lifecycle.plan_retirement_exact(
                    basis,
                    SignalBranchRetirementReason::DependencyCancellation,
                ) {
                    TransitionOutcome::Success(plan) => plan,
                    TransitionOutcome::Denied(_) => {
                        return Err(WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied);
                    }
                };
                let cancellation = SignalOwnerCancellationSource::new();
                match self
                    .signal_lifecycle
                    .retire_exact(plan, &cancellation.token())
                {
                    TransitionOutcome::Success(_) => Ok(()),
                    TransitionOutcome::Denied(_) => {
                        Err(WorthQueryProductBranchOwnerCleanupDenial::SignalOwnerDenied)
                    }
                }
            }
        }
    }
}
