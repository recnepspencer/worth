use super::super::{
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderSessionAffinity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
};
use super::{WorthQueryProviderSessionGrant, WorthQueryProviderSessionReceiver};

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

impl WorthQueryProviderSessionGrant {
    /// Fence the portable grant before physical readmission or preparation.
    pub fn prepare<'run>(
        self,
        receiver: WorthQueryProviderSessionReceiver<'run>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
    ) -> Result<WorthQueryPreparedProviderSession<'run>, WorthQueryProviderSessionFailure> {
        let provider = receiver.plan.preparation_provider();
        self.prepare_with_receiver(phase, phase.world_owner(), &provider, || Ok(receiver))
    }

    pub(in crate::domain_computation) fn prepare_with_receiver<'run>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        provider: &std::sync::Arc<super::super::WorthQueryGraphProviderAnchor>,
        receiver: impl FnOnce() -> Result<
            WorthQueryProviderSessionReceiver<'run>,
            WorthQueryProviderSessionFailure,
        >,
    ) -> Result<WorthQueryPreparedProviderSession<'run>, WorthQueryProviderSessionFailure> {
        self.admission.admit_home_owner(owner)?;
        self.admission.admit_home_owner(phase.world_owner())?;
        let receiver = receiver()?;
        let grant = receiver.plan.provider_grant_control().capture(self)?;
        receiver.plan.require_current_home(phase.world_owner())?;
        if !std::sync::Arc::ptr_eq(provider, &receiver.plan.preparation_provider())
            || grant.contract != receiver.plan.contract
        {
            return Err(WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::TokenNotMintedForPlan,
                WorthQueryProviderSessionProtocolStage::PlanReadmission,
                "provider session grant belongs to another receiver plan",
                grant.counters,
            ));
        }
        receiver.plan.readmit_physical(phase, grant.admission)
    }
}

impl<'run> WorthQueryPreparedProviderSession<'run> {
    pub(in crate::domain_computation::provider_session::protocol) fn prepare_readmitted(
        affinity: WorthQueryProviderSessionAffinity<'run>,
        counters: WorthQueryProviderSessionProtocolCounters,
    ) -> Result<Self, WorthQueryProviderSessionFailure> {
        let mut session = Self { affinity, counters };
        session.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session.affinity.session().provider().prepare_session(
                &session
                    .affinity
                    .session()
                    .token()
                    .view(session.affinity.session().request()),
            )
        }));
        let failure = match invocation {
            Ok(Ok(())) => {
                return Ok(WorthQueryPreparedProviderSession {
                    affinity: session.affinity,
                    counters: session.counters,
                });
            }
            Ok(Err(failure)) => failure.at_stage(
                WorthQueryProviderSessionProtocolStage::SessionPreparation,
                session.counters,
            ),
            Err(_) => WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::ProviderPanicked,
                WorthQueryProviderSessionProtocolStage::SessionPreparation,
                "provider panicked while preparing the readmitted session",
                session.counters,
            ),
        };
        session.counters.called_provider();
        let posture = session.affinity.session_mut().abort_after_failure();
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
