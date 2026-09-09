use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use worth_query_host::facade::domain::{self, AspectValue, FieldKey, InternedString};

use super::{ContactCounters, PanicController};
use crate::contract::TemporalReadyNode;

pub struct Predicate {
    panic: PanicController,
    contacts: ContactCounters,
}

pub struct ReplacementPredicate(Predicate);

impl Predicate {
    pub fn controlled(contacts: ContactCounters) -> (Self, PanicController) {
        let controller = PanicController {
            panic: Arc::new(AtomicBool::new(false)),
        };
        (
            Self {
                panic: controller.clone(),
                contacts,
            },
            controller,
        )
    }
}

impl ReplacementPredicate {
    pub fn controlled(contacts: ContactCounters) -> (Self, PanicController) {
        let (predicate, panic) = Predicate::controlled(contacts);
        (Self(predicate), panic)
    }
}

impl domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode> for Predicate {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.predicate";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        domain::WorthQueryHostProviderHeapRetention::try_from_parts([
            domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                self.panic.panic.as_ref(),
            ),
            domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                self.contacts.predicate.as_ref(),
            ),
            domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                self.contacts.preconditions.as_ref(),
            ),
            domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                self.contacts.projection.as_ref(),
            ),
            domain::WorthQueryHostProviderHeapRetention::arc_allocation_bytes(
                self.contacts.apply.as_ref(),
            ),
        ])
    }

    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        evaluate_predicate(&self.panic, &self.contacts, observation)
    }
}

impl domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode>
    for ReplacementPredicate
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.replacement-predicate";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        domain::WorthQueryHostConditionalPredicateProvider::<TemporalReadyNode>::retained_heap_bytes(
            &self.0,
        )
    }

    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        evaluate_predicate(&self.0.panic, &self.0.contacts, observation)
    }
}

fn evaluate_predicate(
    panic: &PanicController,
    contacts: &ContactCounters,
    observation: domain::WorthQueryConditionalObservationView<'_>,
) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure> {
    assert!(!panic.panic.load(Ordering::SeqCst), "predicate panic");
    contacts.predicate.fetch_add(1, Ordering::SeqCst);
    let ready = observation.dependency(0).is_some_and(|dependency| {
        let domain::WorthQueryConditionalObservedValue::Present(artifact) = dependency.current()
        else {
            return false;
        };
        matches!(
            artifact.field(&FieldKey::new("IntentGateField").unwrap()),
            Some(AspectValue::String(InternedString::Raw(value))) if value == "ready"
        ) && artifact
            .field(&FieldKey::new("IntentEffectField").unwrap())
            .is_none()
    });
    Ok(if ready {
        domain::WorthQueryHostPredicateDecision::Satisfied
    } else {
        domain::WorthQueryHostPredicateDecision::Unsatisfied
    })
}
