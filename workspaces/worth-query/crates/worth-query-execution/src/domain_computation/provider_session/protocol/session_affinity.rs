use std::sync::Arc;

use super::{
    WorthQueryGraphProviderAnchor, WorthQueryProviderExecutionPlanContract,
    WorthQueryProviderProductAffinity, WorthQueryProviderRunBorrow, WorthQueryProviderSessionLease,
    WorthQueryProviderSessionToken, WorthQueryProviderSessionView, WorthQuerySessionBinding,
};

/// The live, inseparable authority for one admitted provider session.
///
/// The protocol owner mints this value only after the provider returns a token
/// for the exact admitted run and plan. It is deliberately move-only: protocol
/// phases can advance the same authority, but cannot copy or reassemble its
/// run, plan, token, and binding axes.
pub(crate) struct WorthQueryProviderSessionAffinity<'run> {
    _run: WorthQueryProviderRunBorrow<'run>,
    contract: WorthQueryProviderExecutionPlanContract,
    product: WorthQueryProviderProductAffinity,
    session: WorthQueryProviderSessionLease<'run>,
    binding: WorthQuerySessionBinding,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryProviderSessionAffinityIdentity(u64);

impl WorthQueryProviderSessionAffinityIdentity {
    pub(super) fn from_token(token: &WorthQueryProviderSessionToken) -> Self {
        Self(token.generation())
    }
}

#[derive(Clone, Copy)]
pub(in crate::domain_computation) struct WorthQueryProviderSessionAffinityView<'session> {
    #[allow(dead_code)]
    identity: WorthQueryProviderSessionAffinityIdentity,
    #[allow(dead_code)]
    plan: &'session WorthQueryProviderExecutionPlanContract,
}

impl<'run> WorthQueryProviderSessionAffinity<'run> {
    pub(super) fn mint(
        request: worth_execution::ExecutionRequest<'run, 'run>,
        run: WorthQueryProviderRunBorrow<'run>,
        contract: WorthQueryProviderExecutionPlanContract,
        product: WorthQueryProviderProductAffinity,
        provider: Arc<WorthQueryGraphProviderAnchor>,
        token: WorthQueryProviderSessionToken,
    ) -> Self {
        let binding = WorthQuerySessionBinding::seal(&token, &contract);
        Self {
            _run: run,
            contract,
            product,
            session: WorthQueryProviderSessionLease::new(request, provider, token),
            binding,
        }
    }

    pub(super) fn plan(&self) -> &WorthQueryProviderExecutionPlanContract {
        &self.contract
    }

    pub(super) fn session(&self) -> &WorthQueryProviderSessionLease<'run> {
        &self.session
    }

    pub(super) fn session_mut(&mut self) -> &mut WorthQueryProviderSessionLease<'run> {
        &mut self.session
    }

    pub(super) fn binding(&self) -> &WorthQuerySessionBinding {
        &self.binding
    }

    pub(super) fn terminal_product(&self) -> super::WorthQueryProviderTerminalProductAffinity {
        self.product.terminal()
    }

    pub(super) fn provider(&self) -> &WorthQueryGraphProviderAnchor {
        self.session.provider()
    }

    pub(super) fn provider_arc(&self) -> Arc<WorthQueryGraphProviderAnchor> {
        self.session.provider_arc()
    }

    pub(super) fn execution_request(&self) -> worth_execution::ExecutionRequest<'run, 'run> {
        self.session.request()
    }

    pub(super) fn provider_session_view(&self) -> WorthQueryProviderSessionView<'_> {
        self.session.token().view(self.session.request())
    }

    pub(in crate::domain_computation) fn view(&self) -> WorthQueryProviderSessionAffinityView<'_> {
        WorthQueryProviderSessionAffinityView {
            identity: WorthQueryProviderSessionAffinityIdentity::from_token(self.session.token()),
            plan: self.plan(),
        }
    }

    pub(in crate::domain_computation) fn terminal_binding(
        &self,
    ) -> super::WorthQueryProviderSessionTerminalBinding {
        super::WorthQueryProviderSessionTerminalBinding::from_affinity(self)
    }
}

impl<'session> WorthQueryProviderSessionAffinityView<'session> {
    #[allow(dead_code)]
    pub(in crate::domain_computation) const fn identity(
        self,
    ) -> WorthQueryProviderSessionAffinityIdentity {
        self.identity
    }

    #[allow(dead_code)]
    pub(in crate::domain_computation) fn plan(
        self,
    ) -> &'session WorthQueryProviderExecutionPlanContract {
        self.plan
    }
}

impl std::fmt::Debug for WorthQueryProviderSessionAffinity<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProviderSessionAffinity")
            .field("plan_identity", &self.contract.identity())
            .field("token_identity", &self.session.token().identity())
            .finish_non_exhaustive()
    }
}
