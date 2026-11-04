use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::two_recoverable_application_commits;

use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchPreparationDenial;

pub struct CountingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CountingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn sibling_publication_substitution_denies_before_transport() {
    let (world, first, second) = two_recoverable_application_commits(185, 186);
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let transport = CountingTransport(AtomicUsize::new(0));
        let substituted = world
            .application
            .observe_committed_dispatch_outbox(
                &active_phase.phase_for_application_for_test(&world.application),
                &first,
            )
            .unwrap()
            .unwrap()
            .with_product_publication_for_test(second.committed_product_publication().clone());
        assert_eq!(
            substituted.relational_runtime_instance_id(),
            first.provider_runtime_instance_id()
        );
        assert_eq!(
            substituted.committed_product_publication().product_branch(),
            first.committed_product_publication().product_branch()
        );
        assert_ne!(
            substituted.commit_reference(),
            substituted
                .committed_product_publication()
                .relational_commit()
        );

        assert_eq!(
            world.application.perform_committed_external_dispatch_for_test(
                &phase.borrowed_request_for_test(),
                &phase.phase_for_application_for_test(&world.application),
                &transport,
                substituted,
                &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope()
            ),
            Err(
                WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied(
                    worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchAttemptDenial::PublicationCommitMismatch
                )
            )
        );
        assert_eq!(transport.0.load(Ordering::Acquire), 0);
    });
}
