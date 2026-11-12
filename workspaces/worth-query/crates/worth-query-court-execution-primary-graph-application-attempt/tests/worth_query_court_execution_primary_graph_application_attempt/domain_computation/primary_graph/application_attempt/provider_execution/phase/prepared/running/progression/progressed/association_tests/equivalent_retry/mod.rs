use crate::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::association_tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn interleaved_retry_reads_original_after_discarding_derived_evidence() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_application_advancement(
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(),
            |active_phase| {
                let host_custody = active_phase.test_host_request_for_test();
                let phase = &active_phase;
                let snapshot_baseline = world
                    .invariant
                    .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
                let (left, right) = equivalent_programs(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    "racing-equivalent",
                );
                let binding = idempotency(139, 140);
                let left = start(phase, &world.application, left, binding);
                let right = start(phase, &world.application, right, binding);
                let both_attempts = world.invariant.active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
                let left = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                let left = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                );
                assert_only_peer_remains(&active_phase.borrowed_request_for_test(), &world, snapshot_baseline, both_attempts);
                (*(*world.application.runtime_for_test()).primary_provider_for_test()).discard_completed_evidence_for_test();
                let right = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                let right = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                );
                assert_eq!(
                    world
                        .invariant
                        .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    snapshot_baseline
                );
                assert_eq!(
                    world.application.provider_session_resource_count_for_test(),
                    0
                );

                let (executed, recovered) = match (left, right) {
                    (
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                        WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered),
                    )
                    | (
                        WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered),
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                    ) => (executed, recovered),
                    unexpected => {
                        panic!("expected one executed and one recovered commit: {unexpected:?}")
                    }
                };
                assert_eq!(executed.commit_reference(), recovered.commit_reference());
                assert_eq!(executed.changed_record_count(), recovered.changed_record_count());
                assert_eq!(executed.outcome_identity(), Some(recovered.outcome_identity()));
                assert_eq!(executed.emitted_effect_count(), recovered.emitted_effect_count());
                assert!(recovered.new_product_head().memory().is_some());
                assert!(recovered.publishing_incarnation().is_none());
                assert!(recovered.world_artifact_digest().is_none());
                assert!(recovered.world_journal_position().is_none());
                let executed_output = executed
                    .outputs_of::<RetentionOutputs>()
                    .and_then(|outputs| outputs.entity::<RetainedAccount>())
                    .expect("fresh receipt retains its actual output role");
                for (entity, _) in executed.committed_changes().entity_changes() {
                    assert!(recovered.semantic_result().semantic().changed_records().contains(
                        &worth_relational::facade::transactions::RecordRef::Entity(entity)));
                }
                assert!(recovered.semantic_result().semantic().changed_records().contains(
                    &worth_relational::facade::transactions::RecordRef::Entity(executed_output.entity_id())));
                assert_eq!(recovered.exact_output_settlement(), worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot { slot: 0 });
                assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
                assert_eq!(executed.terminal().kind(), WorthQueryApplicationCommitTerminalKind::Executed);
            },
        )
        .unwrap();
}

#[test]
fn genuinely_interleaved_equivalent_sessions_validate_retry_cleanup_separately() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world
        .application
        .with_application_advancement(
            &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope(),
            |active_phase| {
                let host_custody = active_phase.test_host_request_for_test();
                let phase = &active_phase;
                let snapshot_baseline = world
                    .invariant
                    .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
                let (left, right) = equivalent_programs(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &world,
                    "genuinely-interleaved-equivalent",
                );
                // The owner reports the two original admission pins before
                // running preparation adds its separate temporary snapshots.
                let admitted_snapshots = world.invariant.active_snapshot_count_for_test(
                    &active_phase.borrowed_request_for_test());
                let admitted_pair = admitted_snapshots - snapshot_baseline;
                assert!(admitted_pair > 0 && admitted_pair.is_multiple_of(2));
                let binding = idempotency(139, 140);
                let left = start(phase, &world.application, left, binding);
                let right = start(phase, &world.application, right, binding);
                let both_attempts = world.invariant.active_snapshot_count_for_test(&active_phase.borrowed_request_for_test());
                assert!(both_attempts >= admitted_snapshots);
                let running_pair = both_attempts - snapshot_baseline;
                assert!(running_pair.is_multiple_of(2));
                let one_unprogressed = running_pair / 2;
                let left = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                // Measure one progressed owner while its equivalent peer still
                // owns its original admission custody. Do not count scratch
                // that progression has already retired as retained custody.
                let one_progressed = world.invariant.active_snapshot_count_for_test(
                    &active_phase.borrowed_request_for_test()) - snapshot_baseline - one_unprogressed;
                assert!(one_progressed > 0);
                let right = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                    None,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                let progressed_pair = snapshot_baseline + 2 * one_progressed;
                assert_eq!(
                    world.invariant.active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    progressed_pair,
                    "both progressed completions retain their actual progressed snapshot custody"
                );
                let left = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                );
                assert_only_peer_remains(&active_phase.borrowed_request_for_test(), &world, snapshot_baseline, progressed_pair);
                let right = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
                );
                assert_eq!(
                    world
                        .invariant
                        .active_snapshot_count_for_test(&active_phase.borrowed_request_for_test()),
                    snapshot_baseline
                );
                assert_eq!(
                    world.application.provider_session_resource_count_for_test(),
                    0
                );

                let (executed, recovered) = match (left, right) {
                    (
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                        WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered),
                    )
                    | (
                        WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered),
                        WorthQueryApplicationCommitOutcome::Committed(executed),
                    ) => (executed, recovered),
                    unexpected => {
                        panic!("expected one executed and one recovered commit: {unexpected:?}")
                    }
                };
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::recorded_receipt::assert_original(&recovered, &executed);
                assert_eq!(executed.commit_reference(), recovered.commit_reference());
                assert_eq!(executed.changed_record_count(), recovered.changed_record_count());
                assert_eq!(executed.outcome_identity(), Some(recovered.outcome_identity()));
                assert_eq!(executed.emitted_effect_count(), recovered.emitted_effect_count());
                assert!(recovered.new_product_head().memory().is_some());
                assert!(recovered.publishing_incarnation().is_none());
                assert!(recovered.world_artifact_digest().is_none());
                assert!(recovered.world_journal_position().is_none());
                let executed_output = executed
                    .outputs_of::<RetentionOutputs>()
                    .and_then(|outputs| outputs.entity::<RetainedAccount>())
                    .expect("fresh receipt retains its actual output role");
                for (entity, _) in executed.committed_changes().entity_changes() {
                    assert!(recovered.semantic_result().semantic().changed_records().contains(
                        &worth_relational::facade::transactions::RecordRef::Entity(entity)));
                }
                assert!(recovered.semantic_result().semantic().changed_records().contains(
                    &worth_relational::facade::transactions::RecordRef::Entity(executed_output.entity_id())));
                assert_eq!(recovered.exact_output_settlement(), worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot { slot: 0 });
                assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
                assert_eq!(executed.terminal().kind(), WorthQueryApplicationCommitTerminalKind::Executed);
            },
        )
        .unwrap();
}
