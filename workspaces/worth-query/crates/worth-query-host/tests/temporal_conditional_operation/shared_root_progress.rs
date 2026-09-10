use std::sync::{mpsc, Mutex};
use std::time::Duration;

use worth_query_host::facade::{domain, primary_graph};

use super::adapters::{ContactCounters, Predicate};
use super::contract::TemporalReadyNode;
use super::schema::IntentIdentityField;
use super::world::{request_scope, CourtroomWorld};

struct ParkedPredicate {
    predicate: Predicate,
    entered: mpsc::Sender<()>,
    resume: Mutex<Option<mpsc::Receiver<()>>>,
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

#[test]
fn shared_host_root_reads_while_real_bridge_conditional_work_is_parked() {
    let contacts = ContactCounters::default();
    let (predicate, panic) = Predicate::controlled(contacts.clone());
    let (entered, entry) = mpsc::channel();
    let (resume, resumption) = mpsc::channel();
    let world = CourtroomWorld::publish_with_predicate(
        "ready",
        0,
        contacts,
        ParkedPredicate {
            predicate,
            entered,
            resume: Mutex::new(Some(resumption)),
        },
        panic,
        None,
        None,
    );
    let application = &world.application;
    let clock = &world.clock;
    std::thread::scope(|scope| {
        let conditional = scope.spawn(|| {
            let outcome = application
                .on_branch(application.current_world())
                .select()
                .unwrap()
                .conditional_clock(clock)
                .unwrap()
                .observe();
            let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
                outcome
            else {
                panic!("the real conditional operation must complete after release")
            };
            assert_eq!(receipt.committed_operation_count(), 1);
            assert_eq!(
                receipt.execution_provenance()[0].signal_decision(),
                Some(primary_graph::WorthQueryConditionalSignalDecision::Eligible)
            );
        });
        let entered = entry.recv_timeout(Duration::from_secs(5));
        if entered.is_err() {
            let _ = resume.send(());
            conditional.join().unwrap();
            panic!("conditional work never reached the real Bridge predicate");
        }
        let (read, completed) = mpsc::channel();
        let reader = scope.spawn(move || {
            let product = application
                .on_branch(application.current_world())
                .select()
                .is_ok();
            let entity = application
                .on_branch(application.current_world())
                .select()
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    IntentIdentityField::reference(),
                    "intent-1".to_string(),
                    &request_scope(),
                    primary_graph::WorthQueryPrincipalResolutionMode::Certification,
                );
            read.send((product, entity.is_ok())).unwrap();
        });
        let progress = completed.recv_timeout(Duration::from_secs(5));
        // Release before assertions so a broken concurrency boundary fails
        // without leaving a scoped worker blocked during unwinding.
        resume.send(()).unwrap();
        conditional.join().unwrap();
        reader.join().unwrap();
        assert_eq!(progress.unwrap(), (true, true));
    });
}
