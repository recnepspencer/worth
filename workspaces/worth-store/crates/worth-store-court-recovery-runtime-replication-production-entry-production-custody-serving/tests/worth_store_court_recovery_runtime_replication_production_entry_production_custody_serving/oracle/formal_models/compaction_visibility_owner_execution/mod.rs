#[allow(unused_imports)]
use crate::compaction_visibility_owner_execution::*;
use std::collections::BTreeSet;

#[test]
fn ordinary_owner_execution_maps_exact_actual_outcomes() {
    let observations = execute_owner_observations();
    let actions = observations
        .iter()
        .copied()
        .map(crate::compaction_visibility_owner_execution::ExecutedOwnerObservation::map)
        .collect::<BTreeSet<_>>();

    assert_eq!(actions, expected_actions());
}
