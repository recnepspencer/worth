pub use std::sync::{mpsc, Mutex};
pub use std::time::Duration;

pub use worth_query_host::facade::{domain, primary_graph};

pub use super::adapters::{ContactCounters, Predicate};
pub use super::contract::TemporalReadyNode;
pub use super::schema::IntentIdentityField;
pub use super::world::{request_scope, CourtroomWorld};

pub struct ParkedPredicate {
    pub predicate: Predicate,
    pub entered: mpsc::Sender<()>,
    pub resume: Mutex<Option<mpsc::Receiver<()>>>,
}

impl domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode> for ParkedPredicate {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.parked-predicate";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        let predicate = domain::WorthQueryHostConditionalPredicateProvider::<TemporalReadyNode>::retained_heap_bytes(&self.predicate)?.bytes();
        domain::WorthQueryHostProviderHeapRetention::try_from_parts([predicate, 4_096])
    }

    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        if let Some(resume) = self.resume.lock().unwrap().take() {
            self.entered.send(()).unwrap();
            resume.recv().unwrap();
        }
        domain::WorthQueryHostConditionalPredicateProvider::<TemporalReadyNode>::evaluate(
            &self.predicate,
            observation,
        )
    }
}
