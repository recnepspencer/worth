#[cfg(feature = "parallel")]
use std::time::Instant;

#[cfg(feature = "parallel")]
use crate::logic::planner::{StageExecutionOutcome, StageExecutor};
#[cfg(feature = "parallel")]
use crate::tests::domains::fintech::{
    compile_financial_locality_world_with_policy, verify_locality_case_with_policy, DensityRatio,
    FinancialWorldDefinition,
};

#[cfg(feature = "parallel")]
use super::throughput_definition::{assert_within_throughput_budget, profiles};

#[cfg(feature = "parallel")]
#[test]
fn throughput_serial_and_parallel_commit_the_same_operational_digest() {
    let started = Instant::now();
    let policy = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile")
        .policy;
    let definition =
        FinancialWorldDefinition::dense_market_close(41, 256, DensityRatio::FourInFive);
    let serial =
        verify_locality_case_with_policy(definition.clone(), 0, policy, StageExecutor::Serial)
            .expect("serial throughput court should settle");
    let parallel =
        verify_locality_case_with_policy(definition, 0, policy, StageExecutor::balanced_parallel())
            .expect("parallel throughput court should settle");

    assert!(
        !serial
            .execution_stage_outcomes()
            .contains(&StageExecutionOutcome::CompletedParallel),
        "the serial control must not report parallel stage dispatch"
    );
    assert!(
        parallel
            .execution_stage_outcomes()
            .contains(&StageExecutionOutcome::CompletedParallel),
        "the parallel parity lane must prove that the production executor admitted parallel stage dispatch"
    );

    assert_eq!(serial.identity_digest(), parallel.identity_digest());
    assert_eq!(
        serial.operational_digest(),
        parallel.operational_digest(),
        "serial and parallel must commit the same operational graph authority"
    );
    assert_eq!(serial.counters(), parallel.counters());
    assert_eq!(
        serial.necessary_evaluation_count(),
        parallel.necessary_evaluation_count()
    );
    assert_within_throughput_budget(started, "serial/parallel operational digest");
}

#[cfg(feature = "parallel")]
#[test]
fn all_profiles_preserve_serial_parallel_operational_digest() {
    let started = Instant::now();
    for profile in profiles() {
        let definition =
            FinancialWorldDefinition::dense_market_close(41, 256, DensityRatio::FourInFive);
        let mut serial_world =
            compile_financial_locality_world_with_policy(definition.clone(), profile.policy)
                .expect("serial profile world compiles");
        let serial_report = serial_world
            .run_locality_performance_sequence(
                8,
                StageExecutor::Serial,
                profile.explicit_observation,
            )
            .expect("serial profile sequence settles");
        let serial_digest = serial_world
            .locality_operational_digest_without_observation_work()
            .expect("serial profile digest derives");

        let mut parallel_world =
            compile_financial_locality_world_with_policy(definition, profile.policy)
                .expect("parallel profile world compiles");
        let parallel_report = parallel_world
            .run_locality_performance_sequence(
                8,
                StageExecutor::balanced_parallel(),
                profile.explicit_observation,
            )
            .expect("parallel profile sequence settles");
        let parallel_digest = parallel_world
            .locality_operational_digest_without_observation_work()
            .expect("parallel profile digest derives");

        assert_eq!(
            serial_digest, parallel_digest,
            "digest drift in {}",
            profile.name
        );
        assert_eq!(
            serial_report.semantic_work_rows, parallel_report.semantic_work_rows,
            "semantic work drift in {}",
            profile.name
        );
        assert!(
            parallel_report.parallel_stage_dispatches > 0,
            "parallel profile {} must dispatch production parallel work",
            profile.name
        );
    }
    assert_within_throughput_budget(started, "six-profile serial/parallel digest");
}
