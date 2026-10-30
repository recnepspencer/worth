#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::axes::*;

#[test]
fn live_leases_admit_one_and_reject_the_next() {
    admit_live_leases(1);
}

#[test]
fn live_leases_admit_eight_and_reject_the_next() {
    admit_live_leases(8);
}

#[test]
fn live_leases_admit_sixty_four_and_reject_the_next() {
    admit_live_leases(64);
}

#[test]
fn retained_roots_admit_one_and_reject_the_next() {
    admit_retained_roots(1);
}

#[test]
fn retained_roots_admit_four_and_reject_the_next() {
    admit_retained_roots(4);
}

#[test]
fn retained_roots_admit_sixteen_and_reject_the_next() {
    admit_retained_roots(16);
}
