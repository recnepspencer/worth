pub(crate) use {crate::checkpoint_crash::CheckpointCrashScenario};










pub(crate) fn run_checkpoint_case(
    scenario_index: usize,
    scenario: CheckpointCrashScenario,
    schedule_seed: u64,
    perturbation_seed: u64,
) {
    let fixture =
        crate::phase_eight_process::checkpoint_crash::case::fixture_setup::prepare(scenario_index, scenario, schedule_seed, perturbation_seed);
    let crash = crate::phase_eight_process::checkpoint_crash::case::crash_frontier::observe(fixture);
    crate::phase_eight_process::checkpoint_crash::case::mutation_proof::prove(&crash);
    let recoveries = crate::phase_eight_process::checkpoint_crash::case::independent_recoveries::run(crash);
    crate::phase_eight_process::checkpoint_crash::case::observer_comparison::compare(&recoveries);
    crate::phase_eight_process::checkpoint_crash::case::teardown::remove(recoveries);
}