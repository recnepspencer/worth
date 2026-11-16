//! Resume the actual original through World, never a new adoption transaction.
use super::*;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome as Original;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(super) fn continue_recovered_adoption(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovery: WorthQueryBranchAdoptionRecovery,
    ) -> Result<WorthQueryBranchAdoptionRecoveryOutcome, WorthQueryBranchAdoptionRecoveryFailure>
    {
        let inspected = match recovery.product.inspect() {
            Ok(inspected) => inspected,
            Err(cause) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(cause),
                    recovery,
                ))
            }
        };
        let expected = inspected.expected_product();
        if expected.branch_identity()
            != self
                .product()
                .publication_binding()
                .observation()
                .branch_identity()
            || expected.lifecycle_incarnation()
                != self
                    .product()
                    .publication_binding()
                    .observation()
                    .lifecycle_incarnation()
        {
            drop(inspected);
            return Err(failure(
                WorthQueryBranchAdoptionRecoveryDenial::ProductAffinityMismatch,
                recovery,
            ));
        }
        let actual = inspected.attempt_identity();
        let attempt =
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
                actual
                    .owner_identity()
                    .durable()
                    .expect("recovered World has durable owner"),
                actual.ordinal(),
            );
        let branch = expected.branch_identity().clone();
        drop(inspected);
        let result = match recovery.product.continue_original_in_request(execution) {
            Ok(result) => result,
            Err(cause) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::Inspection(cause),
                    recovery,
                ))
            }
        };
        let commit = match result {
            Original::Published { commit } => commit,
            Original::Pending(_) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::RecoveredPending,
                    recovery,
                ))
            }
            Original::EvidenceRequired(_) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::RecoveredEvidenceRequired,
                    recovery,
                ))
            }
            Original::Indeterminate(_) => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::RecoveredIndeterminate,
                    recovery,
                ))
            }
            Original::Refused { cause, .. } => {
                return Err(failure(
                    WorthQueryBranchAdoptionRecoveryDenial::RecoveredRefused(cause),
                    recovery,
                ))
            }
        };
        let WorthQueryBranchAdoptionRecovery {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            product,
            support_custody: _,
        } = recovery;
        let adoption = WorthQueryPerformedBranchAdoption::recovered(
            commit,
            branch,
            source,
            target,
            selected_entity_count,
            migration,
            custody,
        );
        let discharge = self
            .application()
            .installed_program_support()
            .expect("readmitted support")
            .lifecycle()
            .release_branch_adoption(execution, attempt);
        let cleanup=self.application().release_product_recovery_in_request(execution,product,IMMEDIATE_RECOVERY_RELEASE_AGE_TICKS,&mut crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements::default());
        let outcome = WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup };
        Ok(match discharge {
            Ok(()) => outcome,
            Err(cause) => WorthQueryBranchAdoptionRecoveryOutcome::ObligationDischargePending {
                cause,
                outcome: Box::new(outcome),
            },
        })
    }
}
