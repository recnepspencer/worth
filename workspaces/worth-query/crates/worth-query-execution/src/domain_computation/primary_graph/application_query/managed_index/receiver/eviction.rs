//! Admitted cleanup uses the real selected Native image, never an old handle root.
use super::super::{
    handle::WorthQueryManagedIndex,
    retention::{ManagedIndexState, WorthQueryManagedIndexLifecycleDenial},
};
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Makes the current maintained material cold under this Parent admission.
    /// Historical Native/World images may retain their immutable temporal shares.
    pub fn evict_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: BoundPlan<'_, S, B>,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
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
            .map_err(lifecycle)?;
            if !index.state.same_association(&candidate) {
                return Err(lifecycle(
                    WorthQueryManagedIndexLifecycleDenial::ForeignAssociation,
                ));
            }
            let graph = self
                .runtime
                .primary_graph()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignContext))?;
            let Some(program) = graph.layout.managed_source(I::NAME) else {
                return index.state.evict().map_err(lifecycle);
            };
            let (epoch, next) = index.state.admit_eviction().map_err(lifecycle)?;
            let retired = graph
                .with_runtime(&execution, |runtime| {
                    runtime.read_truth().evict_managed_index_source(
                        &execution,
                        context.selected.application_basis().snapshot_handle(),
                        program,
                        context.selector.subject(),
                    )
                })
                .map_err(|cause| {
                    ReceiverCause::Provider(ProviderExecutionDenial::NativeSource(cause))
                })?;
            // Runtime admission/locks ended. These two completed cleanups do
            // not consult a request or report a later epoch/disposal failure.
            index.state.finish_eviction(epoch, next);
            drop(retired);
            Ok(())
        })();
        result.map_err(|cause| Denial::new(cause, Some(allocation)))
    }
}
fn lifecycle(cause: WorthQueryManagedIndexLifecycleDenial) -> ReceiverCause {
    ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause))
}
