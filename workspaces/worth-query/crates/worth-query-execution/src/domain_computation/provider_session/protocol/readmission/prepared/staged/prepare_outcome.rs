use super::super::super::super::{
    WorthQueryClosedProviderSessionDisposition, WorthQueryProviderSessionAffinity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture, WorthQuerySessionCommitOrAbortOutcome,
};
use super::WorthQuerySessionBoundReadsAndEffects;

/// Staged session after the provider has accepted commit preparation.
pub struct WorthQuerySessionPrepareOutcome<'run> {
    affinity: WorthQueryProviderSessionAffinity<'run>,
    counters: WorthQueryProviderSessionProtocolCounters,
}

impl std::fmt::Debug for WorthQuerySessionPrepareOutcome<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQuerySessionPrepareOutcome")
            .field("plan_identity", &self.affinity.plan().identity())
            .field("token_identity", &self.affinity.binding().token_identity())
            .finish_non_exhaustive()
    }
}

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    pub(crate) fn prepare_for_commit(
        mut self,
    ) -> Result<WorthQuerySessionPrepareOutcome<'run>, WorthQueryProviderSessionFailure> {
        self.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.affinity
                .session()
                .provider()
                .prepare_staged_session(&self.affinity.session().token().view())
        }));
        let failure = match invocation {
            Ok(Ok(())) => {
                return Ok(WorthQuerySessionPrepareOutcome {
                    affinity: self.affinity,
                    counters: self.counters,
                });
            }
            Ok(Err(failure)) => failure.at_stage(
                WorthQueryProviderSessionProtocolStage::StagedPreparation,
                self.counters,
            ),
            Err(_) => WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::ProviderPanicked,
                WorthQueryProviderSessionProtocolStage::StagedPreparation,
                "provider panicked while preparing staged session work",
                self.counters,
            ),
        };
        self.counters.called_provider();
        let posture = self.affinity.session_mut().abort_after_failure();
        Err(failure.with_recovery_posture(posture))
    }
}

impl WorthQuerySessionPrepareOutcome<'_> {
    pub fn plan_identity(&self) -> &str {
        self.affinity.plan().identity()
    }
    pub fn token_identity(&self) -> &str {
        self.affinity.binding().token_identity()
    }
    pub fn token_generation(&self) -> u64 {
        self.affinity.binding().token_generation()
    }
    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(crate) fn commit(mut self) -> WorthQuerySessionCommitOrAbortOutcome {
        let terminal_binding = self.affinity.terminal_binding();
        self.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.affinity.session_mut().commit()
        }));
        match invocation {
            Ok(Err(
                super::super::super::super::WorthQueryProviderSessionCommitStop::ProductStale(
                    stale,
                ),
            )) => {
                let _ = self.affinity.session_mut().abort();
                WorthQuerySessionCommitOrAbortOutcome::ProductStale(stale)
            }
            Ok(Err(
                super::super::super::super::WorthQueryProviderSessionCommitStop::ProductUnpublished(
                    unpublished,
                ),
            )) => WorthQuerySessionCommitOrAbortOutcome::ProductUnpublished(unpublished),
            Ok(Err(super::super::super::super::WorthQueryProviderSessionCommitStop::NoEffect(
                no_effect,
            ))) => WorthQuerySessionCommitOrAbortOutcome::NoEffect(no_effect),
            Ok(Ok(provider_receipt)) => WorthQuerySessionCommitOrAbortOutcome::Committed(
                WorthQueryClosedProviderSessionDisposition::close(
                    provider_receipt,
                    self.counters,
                    terminal_binding,
                ),
            ),
            Ok(Err(super::super::super::super::WorthQueryProviderSessionCommitStop::Denied(
                failure,
            ))) => WorthQuerySessionCommitOrAbortOutcome::CommitRecoveryRequired(
                failure
                    .at_stage(
                        WorthQueryProviderSessionProtocolStage::Commit,
                        self.counters,
                    )
                    .with_recovery_posture(
                        WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                    ),
            ),
            Ok(Err(super::super::super::super::WorthQueryProviderSessionCommitStop::Deferred(
                deferred,
            ))) => WorthQuerySessionCommitOrAbortOutcome::CommitDeferred(deferred.at_stage(
                WorthQueryProviderSessionProtocolStage::Commit,
                self.counters,
            )),
            Ok(Err(
                super::super::super::super::WorthQueryProviderSessionCommitStop::ControlStopped(
                    stopped,
                ),
            )) => WorthQuerySessionCommitOrAbortOutcome::CommitControlStopped(stopped.at_stage(
                WorthQueryProviderSessionProtocolStage::Commit,
                self.counters,
            )),
            Ok(Err(
                super::super::super::super::WorthQueryProviderSessionCommitStop::SettlementDeferred(
                    deferred,
                ),
            )) => {
                WorthQuerySessionCommitOrAbortOutcome::CommitSettlementDeferred(deferred.at_stage(
                    WorthQueryProviderSessionProtocolStage::Commit,
                    self.counters,
                ))
            }
            Err(_) => WorthQuerySessionCommitOrAbortOutcome::CommitRecoveryRequired(
                WorthQueryProviderSessionFailure::new(
                    WorthQueryProviderSessionDenialKind::ProviderPanicked,
                    WorthQueryProviderSessionProtocolStage::Commit,
                    "provider panicked while committing the prepared session",
                    self.counters,
                )
                .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired),
            ),
        }
    }
}
