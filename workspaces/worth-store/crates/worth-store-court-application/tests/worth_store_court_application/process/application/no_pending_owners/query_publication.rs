//! Request stops retain the real Query adapter's original publication.
use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_query_execution::facade::query_obligations::{
    QueryObligationDenial, QueryObligationFrame, QueryObligationLedger,
    QueryObligationPublicationGenerations, QueryObligationPublicationOutcome as Outcome,
    QueryObligationsMedium, QueryProgramSupportRecord, QUERY_OBLIGATION_FRAME_MAX_BYTES,
};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

struct StoppedQueryPublication {
    fixture: Box<WorldHomeFixture>,
    medium: Arc<dyn QueryObligationsMedium>,
    frame: QueryObligationFrame,
    ledger: QueryObligationLedger,
    generations: QueryObligationPublicationGenerations,
    directory: tempfile::TempDir,
}
impl StoppedQueryPublication {
    fn at(seam: Seam) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let request = crate::workload::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let fixture = Box::new(open(directory.path(), &execution));
        drop(fixture.bootstrap_root(&execution));
        let medium = fixture.home.query_obligations_medium();
        let ledger = QueryObligationLedger::empty(fixture.home.home_identity());
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let frame = ledger
            .prepare(
                QueryProgramSupportRecord::Install {
                    revision: fixture.home.binding().selected().revision(),
                },
                &mut generations,
            )
            .unwrap();
        let gate = fixture
            .home
            .physical_serving_for_test()
            .certification_pause_physical_mutation_at(seam);
        let cancellation = CancellationSource::new();
        let stopped =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        std::thread::scope(|scope| {
            // Only the actual port and original frame cross the worker boundary.
            let worker =
                scope.spawn(|| drive(medium.as_ref(), &frame, &ExecutionRequest::serial(&stopped)));
            let deadline = Instant::now() + Duration::from_secs(30);
            let arrived = loop {
                if gate.await_arrival() {
                    break true;
                }
                if worker.is_finished() || Instant::now() >= deadline {
                    break false;
                }
            };
            cancellation.cancel();
            while !worker.is_finished() && Instant::now() < deadline {
                std::thread::yield_now();
            }
            let returned_while_paused = worker.is_finished();
            // Release even on a failing assertion; a scoped join must not park
            // behind a gate retained by the joining parent.
            gate.release();
            let answer = worker.join().unwrap();
            assert!(
                arrived,
                "the original publication must reach its real physical seam"
            );
            assert!(
                returned_while_paused,
                "the stopped caller returns before physical settlement"
            );
            assert!(
                matches!(
                    answer,
                    Outcome::Refused(QueryObligationDenial::Execution(_))
                ),
                "{answer:?}"
            );
        });
        Self {
            fixture,
            medium,
            frame,
            ledger,
            generations,
            directory,
        }
    }
    fn assert_subject(&self, subject: &worth_proof::HomeQueryObligationSubject) {
        assert_eq!(subject.admit_to(self.medium.authority()), Ok(()));
        assert_eq!(subject.position(), self.frame.position());
        assert_eq!(subject.payload_digest(), self.frame.payload_digest());
    }
    fn assert_one_original_record(&self) {
        let journal = self
            .fixture
            .home
            .physical_serving_for_test()
            .ordered_journal(OrderedJournalName::new("query.obligations").unwrap())
            .unwrap();
        assert_eq!(journal.head().unwrap().unwrap().position(), 1);
        assert_eq!(
            journal
                .read(1, QUERY_OBLIGATION_FRAME_MAX_BYTES)
                .unwrap()
                .into_bytes(),
            self.frame.bytes()
        );
        assert_eq!(
            journal.resolve(2, [0; 32]).unwrap(),
            worth_store::physical_runtime::OrderedJournalResolution::NotCommitted { head: 1 }
        );
    }
    fn close(self) {
        let Self {
            fixture,
            medium,
            frame,
            ledger,
            generations,
            directory,
        } = self;
        drop((medium, frame, ledger, generations));
        let request = crate::workload::serial_request();
        close(fixture.home, &ExecutionRequest::serial(&request));
        drop(directory);
    }
}
fn drive(
    medium: &dyn QueryObligationsMedium,
    frame: &QueryObligationFrame,
    execution: &ExecutionRequest<'_, '_>,
) -> Outcome {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match medium.advance_publication(execution, frame) {
            Outcome::Pending => {
                assert!(
                    Instant::now() < deadline,
                    "the original Query publication must settle"
                );
                std::thread::yield_now();
            }
            terminal @ (Outcome::Committed(_)
            | Outcome::ProvenNoEffect(_)
            | Outcome::Refused(_)
            | Outcome::Unresolved) => return terminal,
        }
    }
}
#[test]
fn a_stopped_query_publication_readmits_its_actual_commit_without_a_second_record() {
    let original = StoppedQueryPublication::at(Seam::AfterRootReplacement);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let Outcome::Committed(proof) = drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("the performed original must hand back committed custody");
    };
    proof.consume(|committed| original.assert_subject(committed.subject()));
    original.assert_one_original_record();
    let head = original
        .fixture
        .home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("query.obligations").unwrap())
        .unwrap()
        .head()
        .unwrap();
    let Outcome::Committed(again) = drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("same-subject handback remains committed");
    };
    again.consume(|committed| original.assert_subject(committed.subject()));
    assert_eq!(
        original
            .fixture
            .home
            .physical_serving_for_test()
            .ordered_journal(OrderedJournalName::new("query.obligations").unwrap())
            .unwrap()
            .head()
            .unwrap(),
        head
    );
    original.close();
}
#[test]
fn a_stopped_query_publication_proves_no_effect_before_an_explicit_fresh_retry() {
    let mut original = StoppedQueryPublication::at(Seam::BeforeEffectCutover);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let Outcome::ProvenNoEffect(proof) =
        drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("the stopped original has a sealed no-effect fate");
    };
    original.assert_subject(proof.subject());
    assert!(original
        .fixture
        .home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("query.obligations").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    original.medium.acknowledge_no_effect(proof).unwrap();
    let Outcome::ProvenNoEffect(repeated) =
        drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("identical bytes retain the original no-effect fate before a fresh commit");
    };
    original.assert_subject(repeated.subject());
    assert!(original
        .fixture
        .home
        .physical_serving_for_test()
        .ordered_journal(OrderedJournalName::new("query.obligations").unwrap())
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    original.medium.acknowledge_no_effect(repeated).unwrap();
    let fresh = original
        .ledger
        .prepare(original.frame.record().clone(), &mut original.generations)
        .unwrap();
    assert_eq!(fresh.position(), original.frame.position());
    assert_eq!(fresh.record(), original.frame.record());
    assert!(fresh.publication_generation() > original.frame.publication_generation());
    assert_ne!(fresh.bytes(), original.frame.bytes());
    original.frame = fresh;
    let Outcome::Committed(proof) = drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("an explicitly retried no-effect request can commit");
    };
    proof.consume(|committed| original.assert_subject(committed.subject()));
    original.assert_one_original_record();
    let Outcome::Committed(repeated) = drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("the fresh committed frame retains one original");
    };
    repeated.consume(|committed| original.assert_subject(committed.subject()));
    original.assert_one_original_record();
    original.close();
}

#[test]
fn a_gracefully_reopened_query_owner_can_publish_after_the_prior_no_effect() {
    let original = StoppedQueryPublication::at(Seam::BeforeEffectCutover);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let Outcome::ProvenNoEffect(proof) =
        drive(original.medium.as_ref(), &original.frame, &execution)
    else {
        panic!("actual original has no effect");
    };
    original.assert_subject(proof.subject());
    original.medium.acknowledge_no_effect(proof).unwrap();
    let StoppedQueryPublication {
        fixture,
        medium,
        frame,
        ledger,
        generations,
        directory,
    } = original;
    let identity = fixture.home.home_identity();
    let old_incarnation = fixture.home.incarnation();
    drop((medium, ledger, generations));
    close(fixture.home, &execution);
    let reopened = Box::new(open(directory.path(), &execution));
    assert_eq!(reopened.home.home_identity(), identity);
    assert_ne!(reopened.home.incarnation(), old_incarnation);
    let medium = reopened.home.query_obligations_medium();
    let ledger = QueryObligationLedger::empty(identity);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    let fresh = ledger
        .prepare(frame.record().clone(), &mut generations)
        .unwrap();
    // This is a new physical lease namespace, not a manufactured Query number.
    // The cold fold is empty because the previous original had no effect.
    assert_eq!(fresh.bytes(), frame.bytes());
    let Outcome::Committed(proof) = drive(medium.as_ref(), &fresh, &execution) else {
        panic!("the real reopened receiver publishes its first obligation");
    };
    let current = StoppedQueryPublication {
        fixture: reopened,
        medium,
        frame: fresh,
        ledger,
        generations,
        directory,
    };
    proof.consume(|committed| current.assert_subject(committed.subject()));
    current.assert_one_original_record();
    current.close();
}
