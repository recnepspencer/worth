//! An in-flight physical dispatch keeps its finite original outbox charge.

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope_for_token;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::inbound::custody::publication_permit::WorthQueryTransportPublicationPermitDenial;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

pub struct BlockingCompletedTransport {
    pub calls: AtomicUsize,
    pub entered: mpsc::SyncSender<([u8; 32], Vec<u8>)>,
    pub release: Mutex<mpsc::Receiver<()>>,
}

impl WorthQueryExternalEffectTransport for BlockingCompletedTransport {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        if self.calls.fetch_add(1, Ordering::AcqRel) == 0 {
            self.entered
                .send((*request.correlation_token(), request.payload().to_vec()))
                .expect("the test observes the physical send");
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .expect("the test releases the physical send");
            WorthQueryExternalTransportOutcome::Completed
        } else {
            WorthQueryExternalTransportOutcome::Acknowledged
        }
    }
}

#[test]
fn callback_terminal_defers_outstanding_capacity_turnover_until_physical_attempt_settles() {
    let world = installed_world();
    {
        let first = world.commit_dispatch(101, "first-outstanding");
        let _second = world.commit_dispatch(102, "second-outstanding");
        let _third = world.commit_dispatch(103, "third-outstanding");
        let owner = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .observe_committed_dispatch_outbox(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &first,
                    )
                    .unwrap()
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let in_flight = (*(*world
            .application
            .runtime_for_test())
            .primary_provider_for_test())
            .begin_external_dispatch_in_flight_for_test(&owner)
            .expect("exact original is committed")
            .expect("installed inbound effect reserves physical attempt");

        let record = first.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0x91; 32], record.payload(), false);
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        assert_eq!(
            world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request)
                .unwrap()
                .posture(),
            WorthQueryInboundReceiptPosture::Performed,
        );
        assert!((*(*world
            .application
            .runtime_for_test())
            .primary_provider_for_test())
            .lookup_completed_inbound_for_test(record.correlation())
            .unwrap()
            .is_some());
        match world.attempt_dispatch_on(
            world.application.current_world(),
            104,
            "capacity-must-remain-charged",
        ) {
            WorthQueryApplicationCommitOutcome::Aborted => {}
            other => {
                panic!("fourth dispatch must be denied at finite outstanding capacity: {other:?}")
            }
        }

        drop(in_flight);
        let fourth = world.commit_dispatch(105, "capacity-after-physical-settlement");
        assert!(fourth.dispatch_outbox().is_some());
    }
}

#[test]
fn callback_winning_during_physical_send_keeps_capacity_until_completed_custody_settles() {
    let world = installed_world();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let transport = Arc::new(BlockingCompletedTransport {
        calls: AtomicUsize::new(0),
        entered: entered_tx,
        release: Mutex::new(release_rx),
    });
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    {
        std::thread::scope(|scope| {
            let first = scope.spawn(|| world.commit_dispatch(111, "physical-send-held"));
            let (token, payload) = entered_rx
                .recv_timeout(Duration::from_secs(10))
                .expect("transport entered after the original outbox committed");
            let _second = world.commit_dispatch(112, "second-outstanding");
            let _third = world.commit_dispatch(113, "third-outstanding");

            let envelope = signed_envelope_for_token(token, [0xA1; 32], &payload, false, 30);
            let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
            let callback = world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request)
                .expect("signed callback can win while physical send has not returned");
            let blocked =
                world.attempt_dispatch_on(world.application.current_world(), 114, "capacity-held");

            release_tx.send(()).unwrap();
            let first_receipt = first.join().expect("physical send worker finishes");
            assert_eq!(
                callback.posture(),
                WorthQueryInboundReceiptPosture::Performed
            );
            assert!(matches!(
                blocked,
                WorthQueryApplicationCommitOutcome::Aborted
            ));
            let first_record = first_receipt.dispatch_outbox().unwrap();
            assert_eq!(first_record.correlation().bytes(), &token);
            assert!((*(*world
                .application
                .runtime_for_test())
                .primary_provider_for_test())
                .lookup_completed_inbound_for_test(first_record.correlation())
                .unwrap()
                .is_some());
            assert!(world
                .commit_dispatch(115, "capacity-after-completed-custody")
                .dispatch_outbox()
                .is_some());
            assert_eq!(transport.calls.load(Ordering::Acquire), 4);
        });
    }
}

#[test]
fn transport_and_authenticated_publications_share_one_installed_operation_limit() {
    let world = installed_world();
    {
        let receipt = world.commit_dispatch(116, "shared-publication-limit");
        let owner = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                world
                    .application
                    .observe_committed_dispatch_outbox(
                        &host_custody.phase_for_application_for_test(&world.application),
                        &receipt,
                    )
                    .unwrap()
                    .unwrap()
            })
            .expect("the fixture read owns its installed host call");
        let binding = world
            .application
            .resolve_installed_transport_completion_binding_for_test(&owner)
            .unwrap();
        let permit = world
            .application
            .reserve_installed_transport_publication_for_test(&binding)
            .unwrap();
        assert!(matches!(
            world
                .application
                .reserve_installed_transport_publication_for_test(&binding),
            Err(WorthQueryTransportPublicationPermitDenial::AtCapacity)
        ));
        let record = receipt.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0xA2; 32], record.payload(), false);
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let accepted = world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap();
        assert_eq!(
            accepted.posture(),
            WorthQueryInboundReceiptPosture::AcceptedPending
        );
        assert_eq!(accepted.pending_reason(), Some(worth_query_execution::facade::domain_computation::WorthQueryInboundPendingReason::PublicationAtCapacity));
        drop(permit);
        assert_eq!(
            world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request)
                .unwrap()
                .posture(),
            WorthQueryInboundReceiptPosture::Performed,
        );
    }
}
