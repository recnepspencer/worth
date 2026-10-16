use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn fintech_recovery_falls_back_from_corrupt_latest_checkpoint_and_keeps_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = setup_world_for(execution, FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(&mut world);
    let _shock = shock_market_on_branch(execution, &mut world, analysis.clone());
    checkpoint_world(execution, &mut world).unwrap();
    let correction = correct_seeded_trade_candidate(execution, &mut world, analysis);
    checkpoint_world(execution, &mut world).unwrap();
    let plan = world
        .runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let corrupted = corrupt_latest_checkpoint_file(&plan);
    assert!(corrupted.is_some());

    let (recovered, outcome) = recover_runtime_from_plan(
        execution,
        world
            .runtime
            .durability()
            .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
            .unwrap(),
    )
    .unwrap();
    let recovered_snapshot = recovered.visibility_authority().snapshot();
    let packet = {
        let context = recovered
            .read_truth()
            .query_plan_context(&recovered_snapshot)
            .expect("recovered correction query plan context");
        PlannedQueryPacket::explicit_targets(
            "correction-probe",
            context,
            world
                .packet_for_correction_probe(&world.latest_snapshot())
                .explicit_target_refs()
                .expect("correction probe uses explicit targets")
                .to_vec(),
        )
    };
    let result = recovered
        .read_truth()
        .execute_query_plan(
            recovered
                .read_truth()
                .plan_query_packet(&recovered_snapshot, packet)
                .expect("planned recovered correction query"),
            execution,
        )
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("executed recovered correction query")
        .result;

    assert_eq!(outcome.latest_commit, Some(correction.commit.clone()));
    assert!(!outcome
        .integrity_report
        .skipped_corrupt_checkpoints
        .is_empty());
    assert!(result
        .entities
        .iter()
        .any(|entity| read_entity_field(entity, field_key("corrected")) == Some("true".into())));
}
