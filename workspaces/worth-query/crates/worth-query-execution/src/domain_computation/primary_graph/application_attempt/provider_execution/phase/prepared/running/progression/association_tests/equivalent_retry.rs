use super::*;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;

#[test]
fn interleaved_retry_reads_original_after_discarding_derived_evidence() {
    let world = installed_authorization_world(true);
    world
        .application
        .with_application_advancement(
            &crate::domain_computation::primary_graph::tests::fixture::live_scope(),
            |active_phase| {
                let host_custody = active_phase.test_host_request();
                let phase = &active_phase;
                let snapshot_baseline = world
                    .invariant
                    .active_snapshot_count(&active_phase.borrowed_request_for_test());
                let (left, right) = equivalent_programs(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    "racing-equivalent",
                );
                let binding = idempotency(139, 140);
                let left = start(phase, &world.application, left, binding);
                let right = start(phase, &world.application, right, binding);
                let both_attempts = world.invariant.active_snapshot_count(&active_phase.borrowed_request_for_test());
                let left = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                    ExecutionAllocationPolicy::SystemAllocation,
                );
                let left = finish_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    left,
                );
                assert_only_peer_remains(&active_phase.borrowed_request_for_test(), &world, snapshot_baseline, both_attempts);
                world.application.primary_provider.discard_completed_evidence_for_test();
                let right = progress_application_commit(
                    &phase.borrowed_request_for_test(),
                    phase,
                    &world.application,
                    right,
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
                        .active_snapshot_count(&active_phase.borrowed_request_for_test()),
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
                assert_eq!(recovered.semantic_result().semantic(), executed.committed_changes().semantic_result());
                assert!(recovered.semantic_result().semantic().changed_records().contains(
                    &worth_relational::facade::transactions::RecordRef::Entity(executed_output.entity_id())));
                assert_eq!(recovered.exact_output_settlement(), worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot { slot: 0 });
                assert_eq!(executed.terminal().attempt_resources_released(), Some(true));
                assert_eq!(executed.terminal().kind(), WorthQueryApplicationCommitTerminalKind::Executed);
            },
        )
        .unwrap();
}
