#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::ready::*;
use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkCapacityDimension,
};

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn ready_commands_admit_one_and_reject_the_next() {
    admit_ready_commands(1);
}

#[test]
fn ready_commands_admit_four_and_reject_the_next() {
    admit_ready_commands(4);
}

#[test]
fn ready_commands_admit_eight_and_reject_the_next() {
    admit_ready_commands(8);
}
