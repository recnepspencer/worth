use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::provider_anchor::{
    WorthQueryErasedProviderSessionLifecycle, WorthQueryGraphProviderAnchor,
    WorthQueryTypedGraphParticipationProvider,
};
use super::semantic_provider_ports::{
    WorthQueryErasedDecisionFactProvider, WorthQueryErasedInvariantExecutionProvider,
    WorthQueryErasedProvisionalGraphProvider,
};
use crate::domain_computation::{
    WorthQueryDecisionFactProvider, WorthQueryGraphParticipationProvider,
    WorthQueryInvariantExecutionProvider, WorthQueryProviderSessionLifecycle,
    WorthQueryProvisionalGraphProvider,
};

static NEXT_GRAPH_PROVIDER_GENERATION: AtomicU64 = AtomicU64::new(1);

impl WorthQueryGraphProviderAnchor {
    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install<G: 'static, P: WorthQueryGraphParticipationProvider<G>>(provider: P) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        Self {
            provider: Arc::new(WorthQueryTypedGraphParticipationProvider::<G, P> {
                provider: Arc::clone(&provider),
                _graph: PhantomData,
            }),
            session_lifecycle: None,
            decision_fact_provider: None,
            provisional_provider: None,
            invariant_provider: None,
            convergence_provider: None,
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_convergent<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G>
            + crate::domain_computation::WorthQueryConvergenceDomainProvider,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let convergence_provider: Arc<
            dyn crate::domain_computation::WorthQueryConvergenceDomainProvider,
        > = provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: None,
            decision_fact_provider: None,
            provisional_provider: None,
            invariant_provider: None,
            convergence_provider: Some(convergence_provider),
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_session_capable<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G> + WorthQueryProviderSessionLifecycle,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let session_lifecycle: Arc<dyn WorthQueryErasedProviderSessionLifecycle> = provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: Some(session_lifecycle),
            decision_fact_provider: None,
            provisional_provider: None,
            invariant_provider: None,
            convergence_provider: None,
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_decision_capable<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G>
            + WorthQueryProviderSessionLifecycle
            + WorthQueryDecisionFactProvider,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let session_lifecycle: Arc<dyn WorthQueryErasedProviderSessionLifecycle> = provider.clone();
        let decision_fact_provider: Arc<dyn WorthQueryErasedDecisionFactProvider> =
            provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: Some(session_lifecycle),
            decision_fact_provider: Some(decision_fact_provider),
            provisional_provider: None,
            invariant_provider: None,
            convergence_provider: None,
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_provisional_capable<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G>
            + WorthQueryProviderSessionLifecycle
            + WorthQueryDecisionFactProvider
            + WorthQueryProvisionalGraphProvider,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let session_lifecycle: Arc<dyn WorthQueryErasedProviderSessionLifecycle> = provider.clone();
        let decision_fact_provider: Arc<dyn WorthQueryErasedDecisionFactProvider> =
            provider.clone();
        let provisional_provider: Arc<dyn WorthQueryErasedProvisionalGraphProvider> =
            provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: Some(session_lifecycle),
            decision_fact_provider: Some(decision_fact_provider),
            provisional_provider: Some(provisional_provider),
            invariant_provider: None,
            convergence_provider: None,
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_invariant_capable<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G>
            + WorthQueryProviderSessionLifecycle
            + WorthQueryDecisionFactProvider
            + WorthQueryProvisionalGraphProvider
            + WorthQueryInvariantExecutionProvider,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let session_lifecycle: Arc<dyn WorthQueryErasedProviderSessionLifecycle> = provider.clone();
        let decision_fact_provider: Arc<dyn WorthQueryErasedDecisionFactProvider> =
            provider.clone();
        let provisional_provider: Arc<dyn WorthQueryErasedProvisionalGraphProvider> =
            provider.clone();
        let invariant_provider: Arc<dyn WorthQueryErasedInvariantExecutionProvider> =
            provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: Some(session_lifecycle),
            decision_fact_provider: Some(decision_fact_provider),
            provisional_provider: Some(provisional_provider),
            invariant_provider: Some(invariant_provider),
            convergence_provider: None,
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }

    #[doc(hidden)]
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn install_convergent_invariant_capable<
        G: 'static,
        P: WorthQueryGraphParticipationProvider<G>
            + WorthQueryProviderSessionLifecycle
            + WorthQueryDecisionFactProvider
            + WorthQueryProvisionalGraphProvider
            + WorthQueryInvariantExecutionProvider
            + crate::domain_computation::WorthQueryConvergenceDomainProvider,
    >(
        provider: P,
    ) -> Self {
        let provider = Arc::new(provider);
        let resource_support = provider.execution_resource_support();
        let session_lifecycle: Arc<dyn WorthQueryErasedProviderSessionLifecycle> = provider.clone();
        let decision_fact_provider: Arc<dyn WorthQueryErasedDecisionFactProvider> =
            provider.clone();
        let provisional_provider: Arc<dyn WorthQueryErasedProvisionalGraphProvider> =
            provider.clone();
        let invariant_provider: Arc<dyn WorthQueryErasedInvariantExecutionProvider> =
            provider.clone();
        let convergence_provider: Arc<
            dyn crate::domain_computation::WorthQueryConvergenceDomainProvider,
        > = provider.clone();
        Self {
            provider: typed_provider::<G, P>(provider),
            session_lifecycle: Some(session_lifecycle),
            decision_fact_provider: Some(decision_fact_provider),
            provisional_provider: Some(provisional_provider),
            invariant_provider: Some(invariant_provider),
            convergence_provider: Some(convergence_provider),
            provider_identity: std::any::type_name::<P>(),
            provider_generation: next_generation(),
            resource_support,
        }
    }
}

fn typed_provider<G: 'static, P: WorthQueryGraphParticipationProvider<G>>(
    provider: Arc<P>,
) -> Arc<WorthQueryTypedGraphParticipationProvider<G, P>> {
    Arc::new(WorthQueryTypedGraphParticipationProvider {
        provider,
        _graph: PhantomData,
    })
}

fn next_generation() -> u64 {
    NEXT_GRAPH_PROVIDER_GENERATION.fetch_add(1, Ordering::Relaxed)
}
