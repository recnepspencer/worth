//! Signed meaning survives a separate transport World terminal until cutoff.

use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope_for_seconds;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::installed_transport::InstalledTransportCompletion;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::inbound_publication::custody::InstalledTransportResumeOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInboundAdmissionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;

pub struct CompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn transport_winner_retains_compact_signed_meaning_until_cutoff() {
    let world = installed_world();
    let receipt = { world.commit_dispatch(121, "compact-message-meaning") };
    let owner = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
                .unwrap()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let correlation = *owner.record().correlation();
    let admitted = world
        .application
        .admit_external_dispatch_attempt_for_test(owner.clone())
        .unwrap();
    let in_flight = (*world.application.primary_provider_for_test())
        .begin_external_dispatch_in_flight_for_test(&owner)
        .unwrap()
        .unwrap();
    let transport = CompletingTransport(AtomicUsize::new(0));
    let completed = dispatch_external_effect(&transport, admitted).unwrap();
    world
        .application
        .record_installed_transport_completion_for_test(
            InstalledTransportCompletion::from_observed_dispatch_for_test(owner, &completed)
                .unwrap(),
            in_flight,
        )
        .unwrap();

    let record = receipt.dispatch_outbox().unwrap();
    let identity = [0xC1; 32];
    let original = signed_envelope_for_seconds(record, identity, record.payload(), false, 1);
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let accepted = world
        .application
        .receive_inbound_occurrence(&world.verifier, &original, &cancelled)
        .unwrap();
    assert_eq!(
        accepted.posture(),
        WorthQueryInboundReceiptPosture::AcceptedPending
    );
    assert_eq!(accepted.pending_reason(), Some(worth_query_execution::facade::domain_computation::WorthQueryInboundPendingReason::ExecutionRequest(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))));

    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world
                    .application
                    .resume_installed_transport_completion_for_test(
                        &phase.borrowed_request_for_test(),
                        phase,
                        &correlation,
                        &request
                    ),
                InstalledTransportResumeOutcome::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &original, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &original, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );

    let altered_cutoff = signed_envelope_for_seconds(record, identity, record.payload(), false, 30);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &altered_cutoff, &request),
        Err(WorthQueryInboundAdmissionDenial::AuthenticatedPermanent(proof))
            if proof.kind() == worth_query_execution::facade::domain_computation::WorthQueryInboundPermanentDenialKind::MessageIdentityConflict
    ));
    let one = NonZeroUsize::new(1).unwrap();
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&world.verifier, one)
            .unwrap()
            .reclaimed(),
        0
    );
    std::thread::sleep(Duration::from_secs(2));
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&world.verifier, one)
            .unwrap()
            .reclaimed(),
        1
    );
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &altered_cutoff, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}
