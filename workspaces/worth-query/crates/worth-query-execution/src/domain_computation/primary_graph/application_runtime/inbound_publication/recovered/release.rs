//! A stale original releases through World before publication becomes retryable.
use super::*;
use worth_runtime_world::facade::{
    WorldRecoveredContinuationDenial, WorldRecoveredReleaseOutcome as Outcome,
};

impl RecordedCompletionContinuation {
    pub(super) fn release_stale<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<RecordedCompletionProgress, WorthQueryRecordedCompletionDenial> {
        use WorthQueryRecordedCompletionDenial as Denial;
        match self.state.clone() {
            OriginalCompletionState::Releasing => {
                let recovery = runtime.runtime.product_runtime.owner.recovery_port();
                let effects = recovery
                    .inspect_recovered_effects(&self.handle)
                    .map_err(Denial::World)?;
                let reservation = runtime
                    .runtime
                    .product_runtime
                    .reserve_owner_cleanup_for_application()
                    .map_err(|_| Denial::Pending)?;
                match crate::domain_computation::primary_graph::home_original_progression::release_original(&recovery, execution, effects) {
                    Outcome::Released(released) => {
                        // World retains this descriptive original in its durable catalog.
                        let (original, cleanup) = released.into_parts();
                        drop(original);
                        let identity = reservation.install_unpublished(cleanup);
                        self.state = OriginalCompletionState::Cleanup(
                            WorthQueryProductBranchOwnerCleanup::new(
                                runtime.runtime.product_runtime.clone(),
                                identity,
                            ),
                        );
                    }
                    Outcome::AdmissionRefused { cause, .. } => return Err(Denial::World(cause)),
                    Outcome::Refused { cause, .. } => {
                        return Err(Denial::Refused(WorldRecoveredContinuationDenial::Durable(
                            cause,
                        )))
                    }
                    Outcome::Pending(_) => return Err(Denial::Pending),
                    Outcome::EvidenceRequired(_) => return Err(Denial::EvidenceRequired),
                    Outcome::Indeterminate(_) => return Err(Denial::Indeterminate),
                }
            }
            OriginalCompletionState::Released => {
                return Ok(RecordedCompletionProgress::StaleReleased)
            }
            OriginalCompletionState::Cleanup(_) => {}
            OriginalCompletionState::Unpublished | OriginalCompletionState::Published(_) => {
                unreachable!("release is an explicit original phase")
            }
        }
        let OriginalCompletionState::Cleanup(cleanup) = &self.state else {
            unreachable!("actual release installed cleanup")
        };
        match cleanup
            .clone()
            .retry_in_request_with_retirements(execution, retirements)
        {
            Ok(_) => {
                self.state = OriginalCompletionState::Released;
                Ok(RecordedCompletionProgress::StaleReleased)
            }
            Err(failed) => {
                self.state = OriginalCompletionState::Cleanup(failed.into_cleanup());
                Err(Denial::Pending)
            }
        }
    }
}
