use super::*;
use crate::facade::durability::RecoveryFailureClass;
use crate::tests::support::expect_durability_failure;

#[test]
fn fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = setup_world_for(execution, FintechScenario::PersistedSmokeBook);
    let analysis = open_analysis_branch(execution, &mut world);
    let _shock = shock_market_on_branch(execution, &mut world, analysis.clone());
    checkpoint_world(execution, &mut world).unwrap();
    let _correction = correct_seeded_trade_candidate(execution, &mut world, analysis);
    checkpoint_world(execution, &mut world).unwrap();
    assert!(damage_filed_checkpoint(&world));

    let recovering =
        crate::facade::runtime::RelationalRuntime::new(world.runtime.config().clone())
            .expect("valid native runtime configuration");
    let medium = world
        .medium
        .as_ref()
        .expect("persisted fintech world owns its durable medium");
    let refused = medium
        .recovery_plan(
            execution,
            &recovering,
            crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect_err("a damaged checkpoint is refused, never skipped");

    assert_eq!(
        expect_durability_failure(refused).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}
