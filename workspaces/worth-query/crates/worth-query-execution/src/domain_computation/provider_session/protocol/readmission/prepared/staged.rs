use std::marker::PhantomData;
use std::sync::Arc;

use super::super::super::{
    WorthQueryClosedProviderSessionDisposition, WorthQueryGraphProviderAnchor,
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderSessionAffinity,
    WorthQueryProviderSessionAffinityView, WorthQueryProviderSessionDenialKind,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionProtocolStage, WorthQueryProviderSessionRecoveryPosture,
    WorthQueryProviderSessionView, WorthQuerySessionBinding, WorthQuerySessionCommitOrAbortOutcome,
};
use super::WorthQueryPreparedProviderSession;

mod prepare_outcome;

pub use prepare_outcome::WorthQuerySessionPrepareOutcome;

/// Session whose reads and effects are bound to the admitted provider token.
/// Only this owner and its commit-preparation child can inspect the fields.
pub struct WorthQuerySessionBoundReadsAndEffects<'run> {
    affinity: WorthQueryProviderSessionAffinity<'run>,
    counters: WorthQueryProviderSessionProtocolCounters,
}

impl std::fmt::Debug for WorthQuerySessionBoundReadsAndEffects<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQuerySessionBoundReadsAndEffects")
            .field("plan_identity", &self.affinity.plan().identity())
            .field(
                "token_identity",
                &self.affinity.session().token().identity(),
            )
            .finish_non_exhaustive()
    }
}

impl<'run> WorthQueryPreparedProviderSession<'run> {
    pub fn bind_reads_and_effects(self) -> WorthQuerySessionBoundReadsAndEffects<'run> {
        WorthQuerySessionBoundReadsAndEffects {
            affinity: self.affinity,
            counters: self.counters,
        }
    }
}

pub struct WorthQuerySessionReadAuthority<'session> {
    binding: &'session WorthQuerySessionBinding,
    plan: &'session WorthQueryProviderExecutionPlanContract,
    provider: &'session WorthQueryGraphProviderAnchor,
    session: WorthQueryProviderSessionView<'session>,
    _invariant: PhantomData<fn(&'session mut ()) -> &'session mut ()>,
}

pub struct WorthQuerySessionEffectAuthority<'session> {
    binding: &'session WorthQuerySessionBinding,
    plan: &'session WorthQueryProviderExecutionPlanContract,
    terminal_binding: super::super::super::WorthQueryProviderSessionTerminalBinding,
    _invariant: PhantomData<fn(&'session mut ()) -> &'session mut ()>,
}

impl WorthQuerySessionBoundReadsAndEffects<'_> {
    pub fn read_authority(&self) -> WorthQuerySessionReadAuthority<'_> {
        WorthQuerySessionReadAuthority {
            binding: self.affinity.binding(),
            plan: self.affinity.plan(),
            provider: self.affinity.provider(),
            session: self.affinity.provider_session_view(),
            _invariant: PhantomData,
        }
    }

    pub fn effect_authority(&self) -> WorthQuerySessionEffectAuthority<'_> {
        WorthQuerySessionEffectAuthority {
            binding: self.affinity.binding(),
            plan: self.affinity.plan(),
            terminal_binding: self.affinity.terminal_binding(),
            _invariant: PhantomData,
        }
    }

    pub fn plan(&self) -> &WorthQueryProviderExecutionPlanContract {
        self.affinity.plan()
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

    pub(crate) fn provisional_binding_identity(&self) -> &str {
        self.affinity.binding().canonical_identity()
    }

    pub(crate) fn provisional_provider(&self) -> &WorthQueryGraphProviderAnchor {
        self.affinity.provider()
    }

    pub(crate) fn provisional_provider_arc(&self) -> Arc<WorthQueryGraphProviderAnchor> {
        self.affinity.provider_arc()
    }

    pub(crate) fn provider_session_view(&self) -> WorthQueryProviderSessionView<'_> {
        self.affinity.provider_session_view()
    }

    pub(in crate::domain_computation) fn provider_session_terminal_binding(
        &self,
    ) -> super::super::super::WorthQueryProviderSessionTerminalBinding {
        self.affinity.terminal_binding()
    }

    /// Joins an application-attempt basis to this exact live staged session.
    ///
    /// The descriptive terminal projection is minted only inside the live
    /// session owner; possessing a copied terminal projection cannot admit an
    /// application attempt.
    pub(in crate::domain_computation) fn bind_application_attempt(
        &self,
        basis: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptBasis,
    ) -> Result<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptAffinity, ()>
    {
        basis.bind_live_session(&self.affinity)
    }

    #[allow(dead_code)]
    pub(in crate::domain_computation) fn provider_session_affinity(
        &self,
    ) -> WorthQueryProviderSessionAffinityView<'_> {
        self.affinity.view()
    }

    pub fn abort(mut self) -> WorthQuerySessionCommitOrAbortOutcome {
        let terminal_binding = self.affinity.terminal_binding();
        self.counters.called_provider();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.affinity.session_mut().abort()
        }));
        match invocation {
            Ok(Ok(provider_receipt)) => WorthQuerySessionCommitOrAbortOutcome::Aborted(
                WorthQueryClosedProviderSessionDisposition::close(
                    provider_receipt,
                    self.counters,
                    terminal_binding,
                ),
            ),
            Ok(Err(failure)) => WorthQuerySessionCommitOrAbortOutcome::AbortRecoveryRequired(
                failure
                    .at_stage(WorthQueryProviderSessionProtocolStage::Abort, self.counters)
                    .with_recovery_posture(
                        WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                    ),
            ),
            Err(_) => WorthQuerySessionCommitOrAbortOutcome::AbortRecoveryRequired(
                WorthQueryProviderSessionFailure::new(
                    WorthQueryProviderSessionDenialKind::ProviderPanicked,
                    WorthQueryProviderSessionProtocolStage::Abort,
                    "provider panicked while aborting the session",
                    self.counters,
                )
                .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired),
            ),
        }
    }
}

impl WorthQuerySessionReadAuthority<'_> {
    pub fn token_identity(&self) -> &str {
        self.binding.token_identity()
    }
    pub fn token_generation(&self) -> u64 {
        self.binding.token_generation()
    }
    pub(crate) fn binding(&self) -> &WorthQuerySessionBinding {
        self.binding
    }
    pub(crate) fn plan(&self) -> &WorthQueryProviderExecutionPlanContract {
        self.plan
    }
    pub(crate) fn provider(&self) -> &WorthQueryGraphProviderAnchor {
        self.provider
    }
    pub(crate) fn session(&self) -> WorthQueryProviderSessionView<'_> {
        self.session
    }
}

impl WorthQuerySessionEffectAuthority<'_> {
    pub fn token_identity(&self) -> &str {
        self.binding.token_identity()
    }
    pub fn token_generation(&self) -> u64 {
        self.binding.token_generation()
    }
    pub(crate) fn binding(&self) -> &WorthQuerySessionBinding {
        self.binding
    }
    pub(crate) fn plan(&self) -> &WorthQueryProviderExecutionPlanContract {
        self.plan
    }
    pub(in crate::domain_computation) fn terminal_binding(
        &self,
    ) -> &super::super::super::WorthQueryProviderSessionTerminalBinding {
        &self.terminal_binding
    }
}
