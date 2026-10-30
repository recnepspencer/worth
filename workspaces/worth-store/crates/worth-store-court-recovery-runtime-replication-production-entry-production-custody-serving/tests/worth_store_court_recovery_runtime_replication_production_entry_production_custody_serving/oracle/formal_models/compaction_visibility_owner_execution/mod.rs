#[allow(unused_imports)]
use crate::compaction_visibility_owner_execution::*;
use std::collections::BTreeSet;

use {
    worth_store_formal_models::map_compaction_observation,
    worth_store_formal_models::map_lsm_maintenance_observation,
    worth_store_formal_models::map_lsm_membership_observation,
    worth_store_formal_models::CompactionVisibilityAction,
    worth_store_formal_models::LsmMaintenanceAction,
    worth_store_formal_models::LsmMaintenanceDenial,
    worth_store_formal_models::LsmMembershipAction, worth_store_formal_models::LsmMembershipDenial,
    worth_store_formal_models::ModeledOutcome,
};
use {
    worth_store_physical_isolation::CompactionCutoverDelta,
    worth_store_physical_isolation::CompactionOwnerCaseObservation,
    worth_store_physical_isolation::CompactionReadInterlockPlan,
};
use {
    worth_store_test_support::harness::observe_lsm_maintenance_owner_cases,
    worth_store_test_support::harness::observe_lsm_owner_cases,
    worth_store_test_support::harness::physical_isolation::compaction::admitted_compaction_plan,
};

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
