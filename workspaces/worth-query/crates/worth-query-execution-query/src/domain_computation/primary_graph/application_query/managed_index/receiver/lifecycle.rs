//! Authenticated explicit open/rebuild use the same typed Parent receiver.
use super::super::{
    execution,
    handle::WorthQueryManagedIndex,
    retention::{
        ManagedIndexState, WorthQueryManagedIndexLifecycle, WorthQueryManagedIndexLifecycleDenial,
    },
};
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    fn open_prepared<'a, B: ApplicationQueryBinding<S>, I: ApplicationManagedIndexBinding<S>>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        plan: &mut BoundPlan<'_, S, B>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<(WorthQueryManagedIndex<S, I>, context::ParentContext<'a, S>), ReceiverCause> {
        let context = context::prepare(execution, self, binding, plan, allocation)?;
        let adapter = self
            .managed_indexes
            .get_admitted::<I>(execution, allocation)?
            .ok_or(ReceiverCause::Refused(Refusal::MissingProvider))?;
        let candidate = ManagedIndexState::new(
            execution,
            context.association.clone(),
            adapter,
            &plan.graph_work,
            &context.selected,
        )
        .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause)))?;
        let graph = self
            .runtime
            .primary_graph()
            .ok_or(ReceiverCause::Refused(Refusal::ForeignContext))?;
        let registry = crate::domain_computation::primary_graph::downcast_managed_views_arc(
            graph.managed_derived_views(),
        );
        let (id, state) = registry
            .register_index(execution, candidate)
            .map_err(|cause| ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause)))?;
        if let Some(program) = graph.layout_ref().managed_source(I::NAME) {
            // Parent admission allows registration/status. Content is still
            // read only after the Source's separate fresh READ in execute.
            let observation = graph
                .with_runtime(execution, |runtime| {
                    runtime.read_truth().observe_managed_index_source_if_ready(
                        execution,
                        context.selected.application_basis().snapshot_handle(),
                        program,
                        context.selector.subject(),
                    )
                })
                .map_err(|cause| {
                    ReceiverCause::Provider(ProviderExecutionDenial::NativeSource(cause))
                })?;
            if let Some(observation) = observation {
                state.attach_maintained(observation).map_err(|cause| {
                    ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause))
                })?;
            }
        }
        let handle = WorthQueryManagedIndex {
            state,
            id,
            registry: Arc::downgrade(&registry),
        };
        Ok((handle, context))
    }

    /// Opens or reuses the same runtime-owned registration without building it.
    /// An invariant-maintained index attaches its matching published image as
    /// Ready; a standalone registration starts Created. Content execution still
    /// requires the Source's independent current READ admission.
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
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Execution(cause), None))?;
        let result = self
            .open_prepared::<B, I>(&execution, binding, &mut plan, &allocation)
            .map(|(handle, _)| handle);
        result.map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(cause, Some(allocation)))
    }

    /// Opens the runtime-owned registration and rebuilds it ONLY when its status
    /// is `Created` (such as after cold reopen). Already-`Ready` registrations
    /// return without reading Source; `Evicted`, `Failed`, `Rebuilding`, and
    /// `Disposed` registrations refuse honestly without hiding errors.
    pub fn ensure_application_managed_index_ready<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
    ) -> Result<WorthQueryManagedIndex<S, I>, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Execution(cause), None))?;
        let result = (|| {
            let (index, context) =
                self.open_prepared::<B, I>(&execution, binding, &mut plan, &allocation)?;
            match index.lifecycle() {
                WorthQueryManagedIndexLifecycle::Ready => Ok(index),
                WorthQueryManagedIndexLifecycle::Created => {
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
                    .map_err(ReceiverCause::Provider)?;
                    Ok(index)
                }
                WorthQueryManagedIndexLifecycle::Rebuilding => {
                    Err(ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(
                        WorthQueryManagedIndexLifecycleDenial::BuildInProgress,
                    )))
                }
                WorthQueryManagedIndexLifecycle::Disposed => {
                    Err(ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(
                        WorthQueryManagedIndexLifecycleDenial::Disposed,
                    )))
                }
                WorthQueryManagedIndexLifecycle::Evicted
                | WorthQueryManagedIndexLifecycle::Failed => {
                    Err(ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(
                        WorthQueryManagedIndexLifecycleDenial::NotReady,
                    )))
                }
            }
        })();
        result.map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(cause, Some(allocation)))
    }

    /// Explicitly rebuilds from the currently admitted complete Source.
    /// Providers execute outside Native and registry locks under this request.
    /// Maintained indexes refill the same selected Native cache used by invariants.
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
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(ReceiverCause::Execution(cause), None))?;
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
        result.map_err(|cause| crate::domain_computation::primary_graph::application_query::managed_index::denial::managed_index_denial(cause, Some(allocation)))
    }
}
