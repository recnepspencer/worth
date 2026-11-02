//! Actual provider and installed-handler destruction preserves runtime custody order.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, IdentityExecutionSchema, ProgramRequiredMutationBinding,
};
use crate::domain_computation::primary_graph::{
    handler::{CandidateWriter, DecisionReader},
    HandlerResult, WorthQueryDecisionPlan,
};
use std::sync::Mutex;

struct Sentinel(Arc<Mutex<Vec<&'static str>>>, bool);
impl Drop for Sentinel {
    fn drop(&mut self) {
        self.0.lock().unwrap().push("provider");
        if self.1 {
            std::panic::panic_any("provider destructor payload");
        }
    }
}

struct RecordedHandler<S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    inner: Arc<dyn OperationHandler<S, B>>,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl<S: ApplicationSchema + 'static, B: ApplicationMutationBinding<S>> OperationHandler<S, B>
    for RecordedHandler<S, B>
{
    fn decide(
        &self,
        input: &B::Input,
        reader: &mut DecisionReader<'_, '_, '_, S, B>,
    ) -> HandlerResult<B::Decision, B::Denial> {
        self.inner.decide(input, reader)
    }
    fn prepare_decision(
        &self,
        input: &B::Input,
        reader: &mut DecisionReader<'_, '_, '_, S, B>,
    ) -> WorthQueryDecisionPlan<B::Decision, B::Denial> {
        self.inner.prepare_decision(input, reader)
    }
    fn build_candidate(
        &self,
        input: &B::Input,
        decision: B::Decision,
        candidate: &mut CandidateWriter<'_, S, B>,
    ) -> HandlerResult<B::Result, B::Denial> {
        self.inner.build_candidate(input, decision, candidate)
    }
}

impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> Drop for RecordedHandler<S, B> {
    fn drop(&mut self) {
        // Record the actual handler's destruction, not registry clearing.
        self.events.lock().unwrap().push("handler");
    }
}

#[test]
fn provider_is_destroyed_before_the_installed_handler() {
    for panic_in_provider in [false, true] {
        type S = IdentityExecutionSchema;
        type B = ProgramRequiredMutationBinding;
        let mut world = installed_authorization_world(true);
        let events = Arc::new(Mutex::new(Vec::new()));
        *world
            .application
            .runtime
            .primary_provider
            .teardown_sentinel
            .lock()
            .unwrap() = Some(Box::new(Sentinel(Arc::clone(&events), panic_in_provider)));
        let registry = &mut world.application.mutation_handlers;
        let original = registry
            .get::<B>()
            .expect("a real installed handler is present");
        registry.entries.get_mut(B::IDENTITY).unwrap().value = Arc::new(TypedHandlerSlot::<S, B> {
            handler: Arc::new(RecordedHandler {
                inner: original,
                events: Arc::clone(&events),
            }),
        });
        assert!(
            registry.get::<B>().is_some(),
            "the typed installed handler remains callable"
        );
        assert!(events.lock().unwrap().is_empty());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(world)));
        if panic_in_provider {
            let payload = result.expect_err("the actual provider destructor panic resumes");
            assert_eq!(
                payload.downcast_ref::<&str>(),
                Some(&"provider destructor payload")
            );
        } else {
            result.expect("ordinary teardown succeeds");
        }
        assert_eq!(*events.lock().unwrap(), ["provider", "handler"]);
    }
}
