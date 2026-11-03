//! Close owned unpublished effects through their original native settlement.
use super::{
    recovery::recovery_failure, WorthQueryProductUnpublishedRecovery as Recovery,
    WorthQueryProductUnpublishedRecoveryReleaseDenial as Denial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
use crate::domain_computation::primary_graph::{
    application_contribution::execution_denial, WorthQueryAdvancementPhase,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::RuntimeWorldRecoveryDenial;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Dispose this exact original partial without new business admission.
    pub fn close_owned_product_publication_recovery(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: Recovery,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, Failure> {
        self.close_product_publication_recovery_in_advancement(phase, recovery)
    }

    /// Close original custody without adopting the failed publication or repeating effects.
    pub(in crate::domain_computation) fn close_product_publication_recovery_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        recovery: Recovery,
    ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, Failure> {
        let recovery =
            match self.release_product_publication_recovery_in_advancement(phase, recovery, 0) {
                Err(Failure::Recovery(failure))
                    if failure.denial()
                        == Denial::World(RuntimeWorldRecoveryDenial::SettlementRequired) =>
                {
                    failure.into_recovery()
                }
                result => return result,
            };
        let recovery = self.settle_recovery_in_advancement(phase, recovery)?;
        // The returned continuation's action list is descriptive. Only actual
        // native release can prove settlement and cleanup eligibility now.
        self.release_product_publication_recovery_in_advancement(phase, recovery, 0)
    }

    fn settle_recovery_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        recovery: Recovery,
    ) -> Result<Recovery, Failure> {
        let execution = match phase.checked_execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(denial) => return Err(recovery_failure(recovery, Denial::Execution(denial))),
        };
        // Borrow this exact original recovery throughout the admitted owner call.
        // Scope/checkpoint/World refusal returns it, including native partial progress.
        let outcome = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    context.checkpoint(1).map_err(|stop| {
                        Denial::Execution(execution_denial::advancement_checkpoint_stop(stop))
                    })?;
                    recovery
                        .continue_owner_settlement()
                        .map(|_| ())
                        .map_err(Denial::World)
                })
        });
        match outcome {
            Ok(Ok((Ok(()), _))) => Ok(recovery),
            Ok(Ok((Err(denial), _))) => Err(recovery_failure(recovery, denial)),
            Ok(Err(denial)) | Err(denial) => Err(recovery_failure(
                recovery,
                Denial::Execution(execution_denial::advancement_scope_stop(denial)),
            )),
        }
    }
}
