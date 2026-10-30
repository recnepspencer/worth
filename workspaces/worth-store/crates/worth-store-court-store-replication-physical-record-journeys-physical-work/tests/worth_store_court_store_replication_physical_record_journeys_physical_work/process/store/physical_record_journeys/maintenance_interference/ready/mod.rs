#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::ready::*;

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
