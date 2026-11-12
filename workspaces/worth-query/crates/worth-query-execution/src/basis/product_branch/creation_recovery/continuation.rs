//! Continue World's original sibling without re-executing either owner effect.
use super::*;
#[derive(Debug)]
pub enum WorthQueryProductBranchCreationContinuation {
    NoEffect(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationNoEffect),
    Performed(crate::basis::WorthQueryProductBranch),
    ReleasedOriginal(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationRelease),
}
impl WorthQueryProductBranchCreationRecovery {
    pub fn continue_in_advancement(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchCreationContinuation,
        WorthQueryProductBranchCreationRecoveryFailure,
    > {
        let execution = match phase.execution_request_for(&self.runtime) {
            Ok(actual) => actual,
            Err(denial) => {
                return Err(WorthQueryProductBranchCreationRecoveryFailure {
                    denial: WorthQueryProductBranchRecoveryDenial::Execution(denial.into()),
                    recovery: self,
                })
            }
        };
        drop(self.effects.take()); // Exclusive World seat refuses any other live inspection.
        let cancellation = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();
        let answer = self.runtime.owner.branch_port().resume_creation(
            &execution,
            &self.handle,
            &cancellation.token(),
        );
        match answer {
            Ok(worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::NoEffect(actual)) => Ok(WorthQueryProductBranchCreationContinuation::NoEffect(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationNoEffect::from_world(actual))),
            Ok(worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::ReleasedOriginal(actual)) =>
                Ok(WorthQueryProductBranchCreationContinuation::ReleasedOriginal(crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationRelease::from_world(actual))),
            Ok(worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::Performed(
                observation,
            )) => Ok(WorthQueryProductBranchCreationContinuation::Performed(crate::basis::WorthQueryProductBranch::from_occurrence(
                observation.lifecycle_incarnation(),
            ))),
            Ok(
                worth_runtime_world::facade::RuntimeWorldBranchCreationOutcome::ProductUnpublished(
                    effects,
                ),
            ) => {
                self.effects = Some(effects);
                Err(WorthQueryProductBranchCreationRecoveryFailure {
                    denial: WorthQueryProductBranchRecoveryDenial::Busy,
                    recovery: self,
                })
            }
            Err(denial) => Err(WorthQueryProductBranchCreationRecoveryFailure {
                denial: map_recovery_denial(denial),
                recovery: self,
            }),
        }
    }
}
