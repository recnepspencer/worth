//! A workflow-enrolled birth stamps its subject's `created_at` from the
//! commit's trusted admission instant, in the same commit that starts the
//! subject's native instance.
use std::time::Duration;

use worth_query_host::facade::primary_graph::WorthQueryFixedRuntimeTimeSource;

use super::at;
use super::birth::{birth, read};
use super::workflow::open;

#[test]
fn one_birth_commit_stamps_admission_time_and_starts_the_instance() {
    let born_at = at(2026, 11, 26, 15, 30);
    let world = open(WorthQueryFixedRuntimeTimeSource::new(born_at));

    // `birth` projects the instance from this commit's own receipt, and
    // `BornAtAdmission` refused any commit whose stamp differs from its
    // `context.admitted_at()`: the body and the invariant saw one instant.
    let instance = birth(&world, 1);
    assert_eq!(instance.branch(), world.definition.branch());

    let request = read(&world, 1);
    assert_eq!((request.id, request.created_at), (1, born_at));
}

#[test]
fn a_later_birth_stamps_the_advanced_instant_and_the_first_keeps_its_own() {
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 26, 15, 30));
    let world = open(clock.clone());
    birth(&world, 1);

    clock
        .advance(Duration::from_secs(90 * 60))
        .expect("the advance is representable");
    birth(&world, 2);

    assert_eq!(read(&world, 1).created_at, at(2026, 11, 26, 15, 30));
    assert_eq!(read(&world, 2).created_at, at(2026, 11, 26, 17, 0));
}
