use crate::logic::planner::StageExecutionOutcome;
use crate::tests::domains::fintech::{
    compile_financial_locality_world_with_policy, verify_locality_case_with_policy, DensityRatio,
    FinancialWorldDefinition,
};
use crate::tests::leased_execution::support::private_authority_process::{
    run_with_private_authority, NativeExecutionEvidence,
};

use super::throughput_definition::profiles;

#[test]
fn throughput_serial_and_parallel_commit_the_same_operational_digest() {
    if run_with_private_authority(
        "tests::performance_profiles::throughput_parity::throughput_serial_and_parallel_commit_the_same_operational_digest",
        NativeExecutionEvidence::ThroughputParity,
    ) {
        return;
    }
    let policy = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile")
        .policy;
    let definition =
        FinancialWorldDefinition::dense_market_close(41, 256, DensityRatio::FourInFive);
    let serial = verify_locality_case_with_policy(definition.clone(), 0, policy, 1)
        .expect("serial throughput court should settle");
    let parallel = verify_locality_case_with_policy(definition, 0, policy, 4)
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
    println!(
        "THROUGHPUT_PARITY parallel_stages={}",
        parallel
            .execution_stage_outcomes()
            .iter()
            .filter(|outcome| **outcome == StageExecutionOutcome::CompletedParallel)
            .count()
    );
}

#[test]
fn all_profiles_preserve_serial_parallel_operational_digest() {
    if run_with_private_authority(
        "tests::performance_profiles::throughput_parity::all_profiles_preserve_serial_parallel_operational_digest",
        NativeExecutionEvidence::ThroughputParity,
    ) {
        return;
    }
    let mut parallel_dispatches = 0;
    let selected_profiles = profiles();
    let profile_count = selected_profiles.len();
    for profile in selected_profiles {
        let definition =
            FinancialWorldDefinition::dense_market_close(41, 256, DensityRatio::FourInFive);
        let mut serial_world =
            compile_financial_locality_world_with_policy(definition.clone(), profile.policy)
                .expect("serial profile world compiles");
        let serial_report = serial_world
            .run_locality_performance_sequence(8, 1, profile.explicit_observation)
            .expect("serial profile sequence settles");
        let serial_digest = serial_world
            .locality_operational_digest_without_observation_work()
            .expect("serial profile digest derives");

        let mut parallel_world =
            compile_financial_locality_world_with_policy(definition, profile.policy)
                .expect("parallel profile world compiles");
        let parallel_report = parallel_world
            .run_locality_performance_sequence(8, 4, profile.explicit_observation)
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
        parallel_dispatches += parallel_report.parallel_stage_dispatches;
    }
    println!(
        "THROUGHPUT_PARITY profiles={profile_count} parallel_dispatches={parallel_dispatches}"
    );
}
