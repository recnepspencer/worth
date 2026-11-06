//! A real independent Signal owner contender against the installed World head.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::basis::WorthQueryProductBranch;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    CompositePublicationIntent, ConsumedCompositePublication, RuntimeWorldPublicationOutcome,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Prepares an actual Signal-only World attempt from this admitted branch.
    /// The returned one-shot action represents the independent component owner,
    /// so it does not acquire Query's executable-package publication gate.
    /// It cannot replace a basis, install a definition or synthesize a receipt.
    #[doc(hidden)]
    pub fn prepare_signal_only_publication_for_test<'runtime>(
        &'runtime self,
        branch: WorthQueryProductBranch,
        request: &'runtime WorthQueryRequestScope,
    ) -> impl FnOnce() -> ConsumedCompositePublication + 'runtime {
        let selected = self
            .runtime
            .product_runtime
            .admit_product_occurrence(branch.occurrence())
            .expect("the contender starts from a genuinely admitted product");
        let binding = selected.publication_binding();
        let control = binding
            .request_control(request)
            .expect("the contender uses its installed clock");
        let publication = self.runtime.product_runtime.owner.publication_port();
        let prepared = publication
            .prepare_with_signal(
                selected.observation().clone(),
                CompositePublicationIntent::with_signal(None),
                control.cancellation(),
                control.deadline(),
            )
            .expect("the independent Signal owner prepares at the selected head");
        move || {
            self.with_application_advancement(request, |phase| {
                let execution = phase
                    .execution_request_for(&self.runtime.product_runtime)
                    .expect("the contender uses this runtime's request execution");
                let outcome = publication.execute_with_signal(
                    execution,
                    prepared,
                    &mut (),
                    control.cancellation(),
                    |_| Ok(()),
                );
                let RuntimeWorldPublicationOutcome::Performed(performed) = outcome else {
                    panic!("the independent Signal contender must win: {outcome:?}")
                };
                performed.consume()
            })
            .expect("the contender's original request remains admitted")
        }
    }
}
