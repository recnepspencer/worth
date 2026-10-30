#[allow(unused_imports)]
use crate::checkpoint_crash::*;
#[test]
fn killed_checkpoint_writer_reopens_and_observes_each_persisted_effect_frontier() {
    let seeds = scenario_seeds();
    let mut schedules = std::collections::BTreeSet::new();
    let mut perturbations = std::collections::BTreeSet::new();
    for (index, (scenario, seed)) in SCENARIOS.into_iter().zip(seeds).enumerate() {
        assert!(
            schedules.insert(seed.schedule),
            "duplicate checkpoint schedule seed"
        );
        assert!(
            perturbations.insert(seed.perturbation),
            "duplicate checkpoint perturbation seed"
        );
        crate::phase_eight_process::checkpoint_crash::case::run_checkpoint_case(
            index,
            scenario,
            seed.schedule,
            seed.perturbation,
        );
    }
    assert_eq!(schedules.len(), SCENARIO_COUNT);
    assert_eq!(perturbations.len(), SCENARIO_COUNT);
}

#[test]
#[ignore = "focused diagnosis of scenario 4; the sixteen-scenario matrix remains required"]
fn candidate_footer_a_checkpoint_crash_diagnostic() {
    const INDEX: usize = 4;
    let scenario = SCENARIOS[INDEX];
    assert_eq!(scenario.id, "c8-checkpoint-footer-a");
    let seed = scenario_seeds()[INDEX];
    crate::phase_eight_process::checkpoint_crash::case::run_checkpoint_case(
        INDEX,
        scenario,
        seed.schedule,
        seed.perturbation,
    );
}
