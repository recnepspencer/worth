use super::fixture::installed_world;
use super::verifier::signed_envelope;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn empty_operation_program_is_rejected_without_inbound_reservation_or_lookup_work() {
    let world = installed_world();
    let before = world
        .application
        .observe_inbound_cost(&world.verifier)
        .unwrap();
    let outcome = world.attempt_no_effect(70);
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
fn signed_receive_selects_one_effect_at_ten_and_one_thousand_unrelated_dispatches() {
    for population in [10_u64, 1_000] {
        let world = installed_world();
        for seed in 0..population {
            world.commit_unrelated_dispatch(seed + 1, &format!("unrelated-{seed}"));
        }
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
