//! Retire only the original primary provider's erased invocation aliases.
use super::provider_anchor::{
    WorthQueryGraphProviderAnchor, WorthQueryTypedGraphParticipationProvider,
};
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryPrimaryGraphProvider, WorthQueryPrimaryLogicalGraph,
};
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;

pub(crate) struct RetiredPrimaryProviderAnchor {
    _identity: &'static str,
    _generation: u64,
    _resources:
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
}

impl WorthQueryGraphProviderAnchor {
    pub(crate) fn preflight_home_retirement(
        &self,
        expected: &Arc<WorthQueryPrimaryGraphProvider>,
    ) -> Result<(), Denial> {
        type Primary = WorthQueryTypedGraphParticipationProvider<
            WorthQueryPrimaryLogicalGraph,
            Arc<WorthQueryPrimaryGraphProvider>,
        >;
        let original = (self.provider.as_ref() as &dyn Any).downcast_ref::<Primary>();
        let matches = original
            .is_some_and(|original| Arc::ptr_eq(original.provider.as_ref(), expected))
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
    pub(crate) fn retire_home_aliases(
        self,
        expected: &Arc<WorthQueryPrimaryGraphProvider>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetiredPrimaryProviderAnchor, (Self, Denial)> {
        if let Err(cause) = self.preflight_home_retirement(expected) {
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

fn provider_matches(value: &dyn Any, expected: &Arc<WorthQueryPrimaryGraphProvider>) -> bool {
    value
        .downcast_ref::<Arc<WorthQueryPrimaryGraphProvider>>()
        .is_some_and(|original| Arc::ptr_eq(original, expected))
}
