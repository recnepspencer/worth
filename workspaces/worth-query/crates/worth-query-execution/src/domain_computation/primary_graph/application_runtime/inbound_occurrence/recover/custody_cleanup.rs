//! Retain exact inbound recovery custody when release or cleanup cannot finish.
use super::*;
use crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn finish_inbound_handoff(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        next: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        retirements: &mut OwnerCleanupRetirements,
    ) -> RecoveryStep {
        match self.release_product_recovery_in_request(execution, old, 0, retirements) {
            Ok(_) => RecoveryStep::Retain(State::Active(next), Ok(Posture::AcceptedPending)),
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::HandoffWorld {
                        old: failed.into_recovery(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::HandoffCleanup {
                        cleanup: failed.into_cleanup(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }

    pub(super) fn finish_inbound_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        terminal: Arc<WorthQueryInboundTerminalOwnerResult>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> RecoveryStep {
        match self.release_product_recovery_in_request(execution, old, 0, retirements) {
            Ok(_) => RecoveryStep::Terminal(terminal),
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::PublishedWorld {
                        old: failed.into_recovery(),
                        terminal,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::PublishedCleanup {
                        cleanup: failed.into_cleanup(),
                        terminal,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }

    pub(super) fn finish_inbound_stale(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        retirements: &mut OwnerCleanupRetirements,
    ) -> RecoveryStep {
        match self.release_product_recovery_in_request(execution, old, 0, retirements) {
            Ok(_) => RecoveryStep::StaleReleased,
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::StaleWorld(failed.into_recovery()),
                    Err(Denial::RecoveryStaleProduct),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::StaleCleanup(failed.into_cleanup()),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }
}
