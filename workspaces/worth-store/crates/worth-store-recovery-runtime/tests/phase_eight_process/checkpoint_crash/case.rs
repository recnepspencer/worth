use super::CheckpointCrashScenario;

#[path = "case/crash_frontier.rs"]
mod crash_frontier;
#[path = "case/fixture_setup.rs"]
mod fixture_setup;
#[path = "case/independent_recoveries.rs"]
mod independent_recoveries;
#[path = "case/mutation.rs"]
mod mutation;
#[path = "case/mutation_proof.rs"]
mod mutation_proof;
#[path = "case/observer_comparison.rs"]
mod observer_comparison;
#[path = "case/oracle.rs"]
mod oracle;
#[path = "case/teardown.rs"]
mod teardown;

pub(super) fn run_checkpoint_case(
    scenario_index: usize,
    scenario: CheckpointCrashScenario,
    schedule_seed: u64,
    perturbation_seed: u64,
) {
    let fixture =
        fixture_setup::prepare(scenario_index, scenario, schedule_seed, perturbation_seed);
    let crash = crash_frontier::observe(fixture);
    mutation_proof::prove(&crash);
    let recoveries = independent_recoveries::run(crash);
    observer_comparison::compare(&recoveries);
    teardown::remove(recoveries);
}
