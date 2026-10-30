#[allow(unused_imports)]
use crate::runtime_authority_pressure_journey::*;

#[test]
fn runtime_authority_pressure_journey_keeps_observation_read_only_and_phase_scoped() {
    if crate::process_death_probe::run_child_mode() {
        return;
    }

    let first = run_pressure_world();
    let replay = run_pressure_world();
    assert_eq!(replay, first);
}
