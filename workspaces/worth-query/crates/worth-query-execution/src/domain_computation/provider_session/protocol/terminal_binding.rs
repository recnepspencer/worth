use super::{
    WorthQueryProviderSessionAffinity, WorthQueryProviderSessionAffinityIdentity,
    WorthQuerySessionBinding,
};

use std::sync::Arc;

/// Immutable proof that one provider session reached a terminal transition.
///
/// The live affinity owner mints this projection. It carries no lease or retry
/// authority, and its fields stay private so a domain sibling cannot assemble
/// a session identity, token binding, and execution plan independently.
#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryProviderSessionTerminalBinding {
    inner: Arc<WorthQueryProviderSessionTerminalBindingInner>,
}

struct WorthQueryProviderSessionTerminalBindingInner {
    affinity: WorthQueryProviderSessionAffinityIdentity,
    session: WorthQuerySessionBinding,
    plan: super::WorthQueryProviderExecutionPlanContract,
    product: super::WorthQueryProviderTerminalProductAffinity,
}

impl WorthQueryProviderSessionTerminalBinding {
    pub(super) fn from_affinity(affinity: &WorthQueryProviderSessionAffinity<'_>) -> Self {
        Self {
            inner: Arc::new(WorthQueryProviderSessionTerminalBindingInner {
                affinity: WorthQueryProviderSessionAffinityIdentity::from_token(
                    affinity.session().token(),
                ),
                session: affinity.binding().clone(),
                plan: affinity.plan().clone(),
                product: affinity.terminal_product(),
            }),
        }
    }

    pub(in crate::domain_computation) fn affinity_identity(
        &self,
    ) -> WorthQueryProviderSessionAffinityIdentity {
        self.inner.affinity
    }

    pub(in crate::domain_computation) fn admits_mutation_run(
        &self,
        session: super::super::WorthQueryGraphWorkSessionIdentity,
        managed_run: super::super::WorthQueryGraphWorkManagedRunIdentity,
        worker: &str,
    ) -> bool {
        self.inner.plan.managed_run_identity() == worker
            && self.inner.plan.graph_work_session_identity() == Some(session.as_u64())
            && self.inner.plan.graph_work_managed_run_identity() == Some(managed_run.as_u64())
    }

    pub(in crate::domain_computation) fn plan(
        &self,
    ) -> &super::WorthQueryProviderExecutionPlanContract {
        &self.inner.plan
    }

    pub(in crate::domain_computation) fn application_product(
        &self,
    ) -> Option<&crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding>{
        self.inner.product.application_product()
    }

    pub(in crate::domain_computation) fn admits_session_view(
        &self,
        session: super::WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.inner.affinity == session.affinity_identity()
            && self.inner.session.token_identity() == session.identity()
            && self.inner.session.token_generation() == session.generation()
            && self.inner.session.provider_identity() == session.provider_identity()
            && self.inner.session.provider_generation() == session.provider_generation()
            && self.inner.plan.provider_identity() == session.provider_identity()
            && self.inner.plan.provider_generation() == session.provider_generation()
            && self.inner.plan.identity() == session.plan_identity()
    }

    pub(in crate::domain_computation) fn same_session(&self, other: &Self) -> bool {
        self == other
    }

    pub(in crate::domain_computation) fn admits_cleanup_binding(
        &self,
        cleanup: &crate::domain_computation::provider_session::WorthQueryProvisionalOverlayCleanupBinding,
    ) -> bool {
        self.inner.affinity == cleanup.affinity_identity()
            && self.inner.session.token_identity() == cleanup.token_identity()
            && self.inner.session.token_generation() == cleanup.token_generation()
            && self.inner.session.provider_identity() == cleanup.provider_identity()
            && self.inner.session.provider_generation() == cleanup.provider_generation()
            && self.inner.plan.identity() == cleanup.plan_identity()
    }
}

impl PartialEq for WorthQueryProviderSessionTerminalBinding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
            || (self.inner.affinity == other.inner.affinity
                && self.inner.session == other.inner.session
                && self.inner.plan == other.inner.plan
                && self.inner.product == other.inner.product)
    }
}

impl Eq for WorthQueryProviderSessionTerminalBinding {}

impl std::fmt::Debug for WorthQueryProviderSessionTerminalBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProviderSessionTerminalBinding")
            .field("affinity", &self.inner.affinity)
            .field("session", &self.inner.session)
            .field("plan", &self.inner.plan)
            .field("product", &self.inner.product)
            .finish()
    }
}
