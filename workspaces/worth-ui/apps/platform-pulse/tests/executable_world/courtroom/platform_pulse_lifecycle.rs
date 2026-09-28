use std::time::{Duration, Instant};

use crate::external_observation::PlatformPulseLifecycleStreamFailure;
use crate::failure_teardown::PulseExecutableWorldFailure;
use crate::installation::CanonicalPlatformPulse;
use crate::product_process::{
    AwaitingFirstFrame, CargoBuiltPlatformPulse, Installed, PulseExecutableWorld,
};

#[test]
fn expired_first_frame_deadline_preserves_primary_failure_and_teardown_disposition() {
    let installed: PulseExecutableWorld<Installed> =
        PulseExecutableWorld::install(CanonicalPlatformPulse::checked_in())
            .unwrap_or_else(|failure| panic!("install hostile world: {failure}"));
    let binary = CargoBuiltPlatformPulse::exact()
        .unwrap_or_else(|failure| panic!("resolve hostile Cargo executable: {failure}"));
    let awaiting: PulseExecutableWorld<AwaitingFirstFrame> = installed
        .launch(binary)
        .unwrap_or_else(|failure| panic!("launch hostile product process: {failure}"));
    let expired = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .expect("representable expired deadline");
    let failure = match awaiting.await_first_frame(expired) {
        Ok(_) => panic!("expired first-frame deadline must fail"),
        Err(failure) => failure,
    };

    assert!(matches!(
        failure.primary(),
        PulseExecutableWorldFailure::Lifecycle(PlatformPulseLifecycleStreamFailure::Deadline)
    ));
    assert_eq!(failure.teardown().forced_process_termination(), Some(true));
    assert_eq!(failure.teardown().lifecycle_reader_joined(), Some(true));
    assert!(failure.teardown().discarded_lifecycle_envelopes().is_some());
    assert_eq!(failure.teardown().installation_removed(), Some(true));
    assert!(failure.teardown().all_owned_resources_released());
    discard_expected_failure_artifact(failure);
}

fn discard_expected_failure_artifact(
    failure: crate::failure_teardown::PulseExecutableWorldFailureReport,
) {
    let artifact = failure.artifact().unwrap_or_else(|artifact_failure| {
        panic!("retain hostile failure artifact: {artifact_failure}")
    });
    let path = artifact.path().to_owned();
    assert!(path.is_dir());
    assert!(path.join("manifest.json").is_file());
    assert!(path.join("source.wui").is_file());
    assert!(artifact.retained_bytes() <= artifact.maximum_bytes());
    let discarded = failure
        .discard_artifact()
        .unwrap_or_else(|artifact_failure| {
            panic!("discard expected hostile artifact: {artifact_failure}")
        });
    assert!(discarded.removed_owned_root());
    assert!(!path.exists());
}
