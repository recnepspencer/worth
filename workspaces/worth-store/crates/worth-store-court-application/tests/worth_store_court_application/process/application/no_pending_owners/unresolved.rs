use super::*;
use worth_store::physical_runtime::{
    certification::CertificationPhysicalMutationCheckpoint, OrderedJournalResolution,
};
use worth_store_application::{
    ApplicationHomeCloseDenial, ApplicationHomeClosePosture,
    ApplicationHomeOwnerAdmissionDenial as Denial,
};

#[test]
fn an_unresolved_first_journal_publication_refuses_even_with_an_empty_durable_head() {
    for name in ["home.transitions", "query.obligations", "query.dispatch"] {
        let directory = tempfile::tempdir().unwrap();
        let request = crate::workload::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let fixture = open(directory.path(), &execution);
        drop(fixture.bootstrap_root(&execution));
        let home = fixture.home;
        let identity = home.home_identity();
        let physical = home.physical_serving_for_test().runtime_identity();
        {
            let serving = home.physical_serving_for_test();
            let gate = serving.certification_pause_physical_mutation_at(
                CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
            );
            std::thread::scope(|scope| {
                let worker = scope.spawn(|| publish_entry(serving, name));
                assert!(
                    gate.await_arrival(),
                    "the actual journal reaches durable data settlement"
                );
                serving.certification_reject_next_candidate_retention_before_effect();
                gate.release();
                assert!(
                    worker.join().unwrap().is_err(),
                    "durable WAL without a selected root is unresolved"
                );
            });
            let journal = serving
                .ordered_journal(OrderedJournalName::new(name).unwrap())
                .expect("the actual retained Home still permits journal inspection");
            assert!(journal.head().unwrap().is_none());
            assert_eq!(
                journal.resolve(1, [0; 32]).unwrap(),
                OrderedJournalResolution::Unresolved
            );
        }
        let refused = home
            .readmit_no_pending_owners(&execution)
            .err()
            .expect("the exact owner gate must refuse unresolved position one");
        if name == "query.obligations" {
            assert!(matches!(refused.reason(), Denial::QueryObligations(worth_query_execution::facade::program_support::QueryProgramSupportDenial::PublicationUnresolved)));
        } else if name == "query.dispatch" {
            assert!(matches!(
                refused.reason(),
                Denial::QueryDispatch(
                    worth_store_application::ApplicationHomeDispatchJournalDenial::PublicationUnresolved
                )
            ));
        } else {
            assert!(matches!(
                refused.reason(),
                Denial::JournalUnresolved {
                    journal: "home.transitions"
                }
            ));
        }
        assert_contended(directory.path(), &execution);
        let home = refused.into_home();
        assert_eq!(home.home_identity(), identity);
        assert_eq!(
            home.physical_serving_for_test().runtime_identity(),
            physical
        );
        let sealed = home
            .seal_world(&execution)
            .unwrap_or_else(|refused| panic!("the actual World seals: {:?}", refused.reason()));
        let custody = match sealed.finish_close(&execution) {
            ApplicationHomeCloseOutcome::CloseRefused {
                reason: ApplicationHomeCloseDenial::PhysicalInspectionRequired,
                posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
            } => custody,
            ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
                panic!("the unresolved physical publication retains inspection custody: {reason:?}")
            }
            ApplicationHomeCloseOutcome::Closed { .. } => {
                panic!("the unresolved physical publication cannot report successful close")
            }
        };
        assert_eq!(custody.home_identity(), identity);
        assert!(custody.inspection().is_some());
        let actual = custody
            .into_physical_close_outcome()
            .unwrap_or_else(|_| panic!("the actual inspection outcome remains owned"));
        assert!(actual.requires_inspection());
        // Disposal releases this actual inspection evidence; it is not a claim
        // that the ambiguous journal publication or Home close succeeded.
        drop(actual);
    }
}
