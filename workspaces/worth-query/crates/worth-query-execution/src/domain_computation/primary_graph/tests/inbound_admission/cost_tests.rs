use std::sync::Arc;
#[cfg(feature = "test-world-operation-control")]
use std::time::{Duration, Instant};

use super::fixture::installed_world;
use super::schema::WideNotifyOperation;
use super::verifier::{signed_envelope, TestVerifier};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;
#[cfg(feature = "test-world-operation-control")]
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn empty_operation_program_denial_has_no_inbound_reservation_or_lookup_work() {
    let world = installed_world();
    let before = world
        .application
        .observe_inbound_cost(&world.verifier)
        .unwrap();
    let outcome = world.attempt_empty_program(70);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Denied(_)),
        "{outcome:?}"
    );
    let after = world
        .application
        .observe_inbound_cost(&world.verifier)
        .unwrap();
    assert_eq!(after, before);
}

#[cfg(feature = "test-world-operation-control")]
#[test]
fn post_preparation_cancellation_releases_inbound_dispatch_reservation() {
    let world = installed_world();
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let pause = world
        .application
        .pause_after_application_candidate_preparation_for_test(
            std::num::NonZeroUsize::new(1).unwrap(),
        );
    let before = world
        .application
        .observe_inbound_cost(&world.verifier)
        .unwrap();
    let commits = super::tests::owner_commits(&world);
    let outcome = std::thread::scope(|threads| {
        let attempt =
            threads.spawn(|| world.attempt_dispatch_with_scope(73, "no-effect", &request));
        assert!(
            pause.wait_until_reached(Duration::from_secs(20)),
            "the real candidate must reach the post-preparation boundary",
        );
        cancellation.cancel();
        pause.release();
        attempt.join().unwrap()
    });
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Cancelled),
        "application maps World cancellation before effects to Cancelled: {outcome:?}",
    );
    let after = world
        .application
        .observe_inbound_cost(&world.verifier)
        .unwrap();
    assert_eq!(
        after.outstanding_dispatches(),
        before.outstanding_dispatches()
    );
    assert_eq!(after.accepted_occurrences(), before.accepted_occurrences());
    assert_eq!(super::tests::owner_commits(&world), commits);
}

#[test]
fn one_and_one_hundred_exact_duplicates_add_only_bounded_verifier_work() {
    for duplicates in [1_u64, 100] {
        let world = installed_world();
        let dispatch = world.commit_dispatch(71, "cost-selected");
        let record = dispatch.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0x77; 32], record.payload(), false);
        let request = super::super::fixture::live_scope();
        let first = world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap();
        assert_eq!(first.posture(), WorthQueryInboundReceiptPosture::Performed);
        let before = world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap();
        let commits = super::tests::owner_commits(&world);
        for _ in 0..duplicates {
            assert_eq!(
                world
                    .application
                    .receive_inbound_occurrence(&world.verifier, &envelope, &request)
                    .unwrap()
                    .posture(),
                WorthQueryInboundReceiptPosture::Performed
            );
        }
        let after = world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap();
        assert_eq!(
            after.verifier_input_bytes() - before.verifier_input_bytes(),
            duplicates * envelope.len() as u64,
        );
        assert_eq!(
            after.custody_key_probes() - before.custody_key_probes(),
            duplicates
        );
        assert_eq!(after.terminal_key_probes(), before.terminal_key_probes());
        assert_eq!(after.outbox_key_probes(), before.outbox_key_probes());
        assert_eq!(
            after.selected_outbox_records(),
            before.selected_outbox_records()
        );
        assert_eq!(
            after.completion_candidate_prepares(),
            before.completion_candidate_prepares()
        );
        assert_eq!(
            after.world_publication_attempts(),
            before.world_publication_attempts()
        );
        assert_eq!(
            after.world_performed_publications(),
            before.world_performed_publications()
        );
        assert_eq!(after.accepted_occurrences(), before.accepted_occurrences());
        assert_eq!(
            after.accepted_charged_bytes(),
            before.accepted_charged_bytes()
        );
        assert_eq!(after.outstanding_dispatches(), 0);
        assert_eq!(super::tests::owner_commits(&world), commits);
    }
}

#[test]
fn signed_receive_selects_one_effect_among_inbound_bound_dispatches() {
    for population in [10_u64, 1_000] {
        let world = installed_world();
        let wide_operation = world
            .application
            .installed_schema()
            .installed_operation(WideNotifyOperation::reference())
            .unwrap();
        let wide_handle = world
            .application
            .install_inbound_occurrence_verifier(&wide_operation, Arc::new(TestVerifier))
            .unwrap();
        for seed in 0..population {
            world.commit_wide_dispatch(seed + 1, &format!("unrelated-inbound-{seed}"));
        }
        assert_eq!(
            world
                .application
                .observe_inbound_cost(&wide_handle)
                .unwrap()
                .outstanding_dispatches(),
            population,
            "the unrelated dispatches must occupy the inbound owner index",
        );
        let dispatch = world.commit_dispatch(72, "cost-selected");
        let record = dispatch.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0x78; 32], record.payload(), false);
        let before = world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap();
        let receipt = world
            .application
            .receive_inbound_occurrence(
                &world.verifier,
                &envelope,
                &super::super::fixture::live_scope(),
            )
            .unwrap();
        assert_eq!(
            receipt.posture(),
            WorthQueryInboundReceiptPosture::Performed
        );
        let after = world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap();
        assert_eq!(
            after.verifier_input_bytes() - before.verifier_input_bytes(),
            envelope.len() as u64
        );
        assert_eq!(
            after.terminal_key_probes() - before.terminal_key_probes(),
            2
        );
        assert_eq!(after.outbox_key_probes() - before.outbox_key_probes(), 1);
        assert_eq!(
            after.selected_outbox_records() - before.selected_outbox_records(),
            1
        );
        assert_eq!(
            after.completion_candidate_prepares() - before.completion_candidate_prepares(),
            1
        );
        assert_eq!(
            after.world_performed_publications() - before.world_performed_publications(),
            1
        );
    }
}
