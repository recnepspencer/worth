use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{PhysicalWorkCapacity, PhysicalWorkCapacityDimension};

use super::super::physical_work::{serving_from_initialization_with_work_profile, work_fixture};

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

fn admit_ready_commands(bound: usize) {
    let parent = tempfile::tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity =
        PhysicalWorkCapacity::new(bound, 1, 1_024, 1024 * 1024, 4 * 1024 * 1024).unwrap();
    let serving = serving_from_initialization_with_work_profile(
        &parent.path().join("store"),
        profile.with_capacity(capacity),
    );
    let submission = serving.physical_read_submission();
    for _ in 0..bound {
        match submission.submit(request.clone()).into_raw() {
            TransitionOutcome::Success(_) => {}
            other => panic!("{bound} ready commands must admit, saw {other:?}"),
        }
    }
    let before = serving.media_counters();
    assert!(matches!(
        submission.submit(request).into_raw(),
        TransitionOutcome::Deferred(deferred)
            if deferred.capacity() == bound
                && deferred.dimension() == PhysicalWorkCapacityDimension::Commands
    ));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}
