use super::super::{
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderSessionAffinity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
};
use super::WorthQueryProviderPlanReadmission;

mod staged;

pub use staged::{
    WorthQuerySessionBoundReadsAndEffects, WorthQuerySessionEffectAuthority,
    WorthQuerySessionPrepareOutcome, WorthQuerySessionReadAuthority,
};

/// Provider session after physical preparation has succeeded. Its fields are
/// private to this phase owner and its consuming child transition.
pub struct WorthQueryPreparedProviderSession<'run> {
    affinity: WorthQueryProviderSessionAffinity<'run>,
    counters: WorthQueryProviderSessionProtocolCounters,
}

impl std::fmt::Debug for WorthQueryPreparedProviderSession<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryPreparedProviderSession")
            .field("plan_identity", &self.affinity.plan().identity())
            .field(
                "token_identity",
                &self.affinity.session().token().identity(),
            )
            .finish_non_exhaustive()
    }
}

impl<'run> WorthQueryProviderPlanReadmission<'run> {
    pub fn prepare(
        mut self,
    ) -> Result<WorthQueryPreparedProviderSession<'run>, WorthQueryProviderSessionFailure> {
        self.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.affinity.session().provider().prepare_session(
                &self
                    .affinity
                    .session()
                    .token()
                    .view(self.affinity.session().request()),
            )
        }));
        let failure = match invocation {
            Ok(Ok(())) => {
                return Ok(WorthQueryPreparedProviderSession {
                    affinity: self.affinity,
                    counters: self.counters,
                });
            }
            Ok(Err(failure)) => failure.at_stage(
                WorthQueryProviderSessionProtocolStage::SessionPreparation,
                self.counters,
            ),
            Err(_) => WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::ProviderPanicked,
                WorthQueryProviderSessionProtocolStage::SessionPreparation,
                "provider panicked while preparing the readmitted session",
                self.counters,
            ),
        };
        self.counters.called_provider();
        let posture = self.affinity.session_mut().abort_after_failure();
        Err(failure.with_recovery_posture(posture))
    }
}

impl WorthQueryPreparedProviderSession<'_> {
    pub fn plan(&self) -> &WorthQueryProviderExecutionPlanContract {
        self.affinity.plan()
    }

    pub fn token_identity(&self) -> &str {
        self.affinity.session().token().identity()
    }

    pub fn token_generation(&self) -> u64 {
        self.affinity.session().token().generation()
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }
}
