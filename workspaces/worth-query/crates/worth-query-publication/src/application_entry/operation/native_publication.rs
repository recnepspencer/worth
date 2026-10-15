//! Shared native publication custody. Domain delivery and command meaning remain outside.
use super::{WorthQueryOperationCleanup, WorthQueryOperationCloseDenial};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationUncommitted, WorthQueryManagedApplicationRecoveryOutcome as Recovery,
    WorthQueryManagedApplicationRecoveryPerformed,
};

// Private to Publication: no consumer can supply or implement a progression backend.
pub(in crate::application_entry) enum NativePublicationState<Outcome, Completion, AdmissionDenial> {
    Initial,
    Unpublished(WorthQueryProductUnpublishedRecovery),
    Performed(WorthQueryManagedApplicationRecoveryPerformed),
    Keyed(WorthQueryApplicationUncommitted),
    UnresolvedAdvancement(WorthQueryAdvancementDenial),
    RetryableNoEffect(Outcome),
    Stopped(Outcome),
    AdmissionStopped(AdmissionDenial),
    RetainedPending(Completion),
}

pub(in crate::application_entry) struct NativeRecoveredPublication {
    pub receipt: WorthQueryApplicationCommitReceipt,
    pub publication: Option<WorthQueryManagedApplicationRecoveryPerformed>,
    pub initial_posture: Option<WorthQueryApplicationUncommitted>,
    pub initial_advancement_failure: Option<WorthQueryAdvancementDenial>,
}
pub(in crate::application_entry) enum NativeKeyedProgress {
    Progress {
        proved_abort: Option<WorthQueryApplicationUncommitted>,
    },
    Completed(NativeRecoveredPublication),
    Terminal(Resolution),
}

impl<Outcome, Completion, AdmissionDenial>
    NativePublicationState<Outcome, Completion, AdmissionDenial>
{
    pub(in crate::application_entry) fn record_unresolved_advancement(
        &mut self,
        denial: WorthQueryAdvancementDenial,
    ) {
        if matches!(denial, WorthQueryAdvancementDenial::Panicked)
            && matches!(self, Self::Initial | Self::RetryableNoEffect(_))
        {
            *self = Self::UnresolvedAdvancement(denial);
        }
    }

    /// Only these native postures require unpublished recovery or keyed settlement.
    /// Other no-effect/terminal classifications remain with their native outcome owner.
    pub(in crate::application_entry) fn accept_uncommitted(
        &mut self,
        posture: WorthQueryApplicationUncommitted,
    ) -> Result<(), WorthQueryApplicationUncommitted> {
        match posture {
            WorthQueryApplicationUncommitted::ProductUnpublished(partial) => {
                *self = Self::Unpublished(partial.into_recovery());
                Ok(())
            }
            posture @ (WorthQueryApplicationUncommitted::Aborted
            | WorthQueryApplicationUncommitted::Indeterminate(_)) => {
                *self = Self::Keyed(posture);
                Ok(())
            }
            other => Err(other),
        }
    }

    /// NoEffect leaves the original recovery owner intact. Replacement follows native custody.
    pub(in crate::application_entry) fn accept_recovery(
        &mut self,
        outcome: Recovery,
        cleanup: &mut WorthQueryOperationCleanup,
    ) {
        match outcome {
            Recovery::NoEffect => {}
            Recovery::ProductUnpublished {
                next,
                prior_cleanup_failure,
            } => {
                cleanup.absorb_release_failure(prior_cleanup_failure);
                *self = Self::Unpublished(next);
            }
            Recovery::Performed(performed) => *self = Self::Performed(performed),
        }
    }

    /// Resolving a key transfers original publication evidence, never a synthetic result.
    pub(in crate::application_entry) fn accept_keyed(
        &mut self,
        resolution: Resolution,
        cleanup: &WorthQueryOperationCleanup,
    ) -> Result<NativeKeyedProgress, WorthQueryOperationCloseDenial> {
        match resolution {
            Resolution::AlreadyCommitted(receipt) => {
                let prior = std::mem::replace(self, Self::Initial);
                let (publication, initial_posture, initial_advancement_failure) = match prior {
                    Self::Performed(performed) => (Some(performed), None, None),
                    Self::Keyed(posture) => (None, Some(posture), None),
                    Self::UnresolvedAdvancement(failure) => (None, None, Some(failure)),
                    _ => unreachable!("only native performed or unresolved custody resolves a key"),
                };
                Ok(NativeKeyedProgress::Completed(NativeRecoveredPublication {
                    receipt,
                    publication,
                    initial_posture,
                    initial_advancement_failure,
                }))
            }
            Resolution::Unseen
                if matches!(self, Self::Keyed(WorthQueryApplicationUncommitted::Aborted)) =>
            {
                if cleanup.managed_failure().is_some() {
                    return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
                }
                // Native Aborted proves no effect. Unseen by itself never permits execution.
                let Self::Keyed(posture) = std::mem::replace(self, Self::Initial) else {
                    unreachable!("actual native abort guarded by state match")
                };
                Ok(NativeKeyedProgress::Progress {
                    proved_abort: Some(posture),
                })
            }
            Resolution::Unseen => Ok(NativeKeyedProgress::Progress { proved_abort: None }),
            resolution @ Resolution::IntentDrift => Ok(NativeKeyedProgress::Terminal(resolution)),
        }
    }
}

/// Completed no-effect evidence alone permits a new native attempt.
/// Failed Stale/Denied cleanup is classified natively as Indeterminate instead.
pub(in crate::application_entry) fn native_retry_eligible(
    posture: &WorthQueryApplicationUncommitted,
) -> bool {
    use worth_query_execution::facade::primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryInvariantExecutionDenialKind as InvariantKind,
    };
    match posture {
        WorthQueryApplicationUncommitted::Stale(_) => true,
        WorthQueryApplicationUncommitted::Denied(cause) => {
            cause.kind() == Kind::ProductBasisStale
                && cause.stage() == Stage::InvariantExecution
                && cause
                    .invariant_execution_failure()
                    .is_some_and(|failure| failure.kind() == InvariantKind::ProductBasisStale)
        }
        _ => false,
    }
}

/// Disposal eligibility is not effect retry or proof that managed cleanup completed.
pub(in crate::application_entry) fn native_terminal_can_close(
    posture: &WorthQueryApplicationUncommitted,
) -> bool {
    use WorthQueryApplicationUncommitted as Native;
    matches!(
        posture,
        Native::Denied(_)
            | Native::Cancelled
            | Native::TimedOut
            | Native::Stale(_)
            | Native::ProductStale(_)
            | Native::NoEffect(_)
    )
}
