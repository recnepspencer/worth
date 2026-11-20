//! Retire only the original primary provider's erased invocation aliases.
use super::provider_anchor::{
    WorthQueryGraphProviderAnchor, WorthQueryTypedGraphParticipationProvider,
};
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementOwner as Owner;
use crate::ports::RuntimeHomeRetirementDenial as Denial;
use std::any::Any;
use worth_execution::ExecutionRequest;

pub struct RetiredPrimaryProviderAnchor {
    _identity: &'static str,
    _generation: u64,
    _resources:
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
}

impl WorthQueryGraphProviderAnchor {
    pub fn preflight_home_retirement<
        G: Any + Send + Sync,
        P: Any + Send + Sync + PartialEq,
    >(
        &self,
        expected: &P,
    ) -> Result<(), Denial> {
        type Primary<G, P> = WorthQueryTypedGraphParticipationProvider<G, P>;
        let original = (self.provider.as_ref() as &dyn Any).downcast_ref::<Primary<G, P>>();
        let matches = original.is_some_and(|original| original.provider.as_ref() == expected)
            && self
                .session_lifecycle
                .as_ref()
                .is_some_and(|p| provider_matches(p.as_ref(), expected))
            && self
                .decision_fact_provider
                .as_ref()
                .is_some_and(|p| provider_matches(p.as_ref(), expected))
            && self
                .provisional_provider
                .as_ref()
                .is_some_and(|p| provider_matches(p.as_ref(), expected))
            && self
                .invariant_provider
                .as_ref()
                .is_some_and(|p| provider_matches(p.as_ref(), expected))
            && self.convergence_provider.is_none();
        if matches {
            Ok(())
        } else {
            Err(Denial::Unsupported(Owner::GraphProvider))
        }
    }

    /// The separately borrowed original provider keeps every released share
    /// nonfinal. No supplier callback or arbitrary provider destructor runs.
    pub fn retire_home_aliases<G: Any + Send + Sync, P: Any + Send + Sync + PartialEq>(
        self,
        expected: &P,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetiredPrimaryProviderAnchor, (Self, Denial)> {
        if let Err(cause) = self.preflight_home_retirement::<G, P>(expected) {
            return Err((self, cause));
        }
        if let Err(cause) = request.consult() {
            return Err((self, Denial::Execution(cause)));
        }
        let Self {
            session_grant_control,
            provider,
            session_lifecycle,
            decision_fact_provider,
            provisional_provider,
            invariant_provider,
            convergence_provider,
            provider_identity,
            provider_generation,
            resource_support,
        } = self;
        drop(session_grant_control);
        // These exact concrete wrappers contain only their original provider
        // Arc and PhantomData. Preflight checked their pointee, not a label.
        drop((
            provider,
            session_lifecycle,
            decision_fact_provider,
            provisional_provider,
            invariant_provider,
            convergence_provider,
        ));
        Ok(RetiredPrimaryProviderAnchor {
            _identity: provider_identity,
            _generation: provider_generation,
            _resources: resource_support,
        })
    }
}

fn provider_matches<P: Any + Send + Sync + PartialEq>(value: &dyn Any, expected: &P) -> bool {
    value
        .downcast_ref::<P>()
        .is_some_and(|original| original == expected)
}
