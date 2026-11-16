//! Release the exact blocked branch and cancel only the unstarted suffix.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn release_branch_set_adoption_recovery(
        &self,
        recovery: WorthQueryBranchSetAdoptionRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        (
            WorthQueryBranchSetAdoptionCancellation,
            WorthQueryProductBranchOwnerCleanupReceipt,
        ),
        WorthQueryBranchSetAdoptionRecoveryReleaseFailure,
    > {
        let mut retirements = OwnerCleanupRetirements::default();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let (branch, adoption, recovery) = recovery.into_parts();
        match self.release_branch_adoption_custody(
            &worth_execution::ExecutionRequest::serial(&host),
            &mut retirements,
            recovery,
            minimum_age_ticks,
        ) {
            Ok(receipt) => Ok((cancel_unstarted(adoption), receipt)),
            Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::ObligationDischargePending {
                cause,
                cleanup,
            }) => Err(
                WorthQueryBranchSetAdoptionRecoveryReleaseFailure::ObligationDischargePending {
                    cause,
                    cancellation: cancel_unstarted(adoption),
                    cleanup,
                },
            ),
            Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::Recovery { denial, recovery }) => {
                Err(
                    WorthQueryBranchSetAdoptionRecoveryReleaseFailure::Recovery {
                        denial,
                        recovery: WorthQueryBranchSetAdoptionRecovery::retained(
                            branch, adoption, recovery,
                        ),
                    },
                )
            }
            Err(WorthQueryBranchAdoptionRecoveryReleaseFailure::OwnerCleanup(failure)) => Err(
                WorthQueryBranchSetAdoptionRecoveryReleaseFailure::OwnerCleanup {
                    cancellation: cancel_unstarted(adoption),
                    failure,
                },
            ),
        }
    }
}

fn cancel_unstarted(
    adoption: WorthQueryPreparedBranchSetAdoption,
) -> WorthQueryBranchSetAdoptionCancellation {
    WorthQueryBranchSetAdoptionCancellation {
        cancelled_branch_count: adoption.pending.len() + 1,
        progress: adoption.progress,
        total_selection_work_units: adoption.total_selection_work_units,
    }
}
