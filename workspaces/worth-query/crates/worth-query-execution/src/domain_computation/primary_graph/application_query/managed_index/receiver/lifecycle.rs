//! Authenticated explicit open/rebuild use the same typed Parent receiver.
use super::super::{
    execution,
    handle::WorthQueryManagedIndex,
    retention::{ManagedIndexState, WorthQueryManagedIndexLifecycleDenial},
};
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Opens or reuses the same runtime-owned registration, initially Created.
    /// This performs no Source read or provider build.
    pub fn open_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: BoundPlan<'_, S, B>,
    ) -> Result<WorthQueryManagedIndex<S, I>, Denial> {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::new(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| Denial::new(ReceiverCause::Execution(cause), None))?;
        let result = (|| {
            let context = context::prepare(&execution, self, binding, &mut plan, &allocation)?;
            let adapter = self
                .managed_indexes
                .get_admitted::<I>(&execution, &allocation)?
                .ok_or(ReceiverCause::Refused(Refusal::MissingProvider))?;
            let candidate = ManagedIndexState::new(
                &execution,
                context.association,
                adapter,
                &plan.graph_work,
                &context.selected,
            )
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause)))?;
            let graph = self
                .runtime
                .primary_graph()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignContext))?;
            let registry = graph.managed_derived_views();
            let (id, state) = registry
                .register_index(&execution, candidate)
                .map_err(|cause| {
                    ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause))
                })?;
            Ok(WorthQueryManagedIndex {
                state,
                id,
                registry: Arc::downgrade(&registry),
            })
        })();
        result.map_err(|cause| Denial::new(cause, Some(allocation)))
    }

    /// Explicitly rebuilds from the currently admitted complete Source.
    /// Providers execute outside Native and registry locks under this request.
    pub fn rebuild_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), Denial> {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::new(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| Denial::new(ReceiverCause::Execution(cause), None))?;
        let result = (|| {
            let context = context::prepare(&execution, self, binding, &mut plan, &allocation)?;
            let adapter = self
                .managed_indexes
                .get_admitted::<I>(&execution, &allocation)?
                .ok_or(ReceiverCause::Refused(Refusal::MissingProvider))?;
            let candidate = ManagedIndexState::new(
                &execution,
                context.association,
                adapter,
                &plan.graph_work,
                &context.selected,
            )
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause)))?;
            if !index.state.same_association(&candidate) {
                return Err(ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(
                    WorthQueryManagedIndexLifecycleDenial::ForeignAssociation,
                )));
            }
            execution::rebuild(
                &execution,
                self,
                &index.state,
                &plan.graph_work,
                &context.selected,
                &context.selector,
                external,
                plan.controls.request_scope(),
                &allocation,
            )
            .map_err(ReceiverCause::Provider)
        })();
        result.map_err(|cause| Denial::new(cause, Some(allocation)))
    }
}
