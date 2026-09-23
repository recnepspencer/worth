use std::time::{Duration, Instant};

use crate::external_observation::PlatformPulseLifecycleStreamFailure;
use crate::failure_teardown::PulseExecutableWorldFailure;
use crate::installation::CanonicalPlatformPulse;
use crate::product_process::{
    AwaitingFirstFrame, AwaitingReplacement, CargoBuiltPlatformPulse, FirstCurrent, IdentityTraced,
    InitialBlue, Installed, OverlayCleared, OverlayPublished, Published, PulseExecutableWorld,
    SecondCurrent, SnapshotCaptured, WatchedPulseObservationFailure, WatchedPulseTransition,
};
use crate::source_delta::{
    GreenPulseSourceDelta, PulseCausalActionManifest, QueryStatusV1, QueryStatusV2,
};

const TRANSITION_DEADLINE: Duration = Duration::from_secs(5);
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

#[test]
#[ignore = "3.15 snapshot-gated green-source journey; current 3.16 imported-source denial/recovery replaces this evidence"]
fn expired_green_observation_preserves_action_failure_and_teardown_disposition() {
    let canonical = CanonicalPlatformPulse::checked_in();
    let green_delta = GreenPulseSourceDelta::from_checked_in(canonical)
        .unwrap_or_else(|failure| panic!("derive hostile green delta: {failure}"));
    let installed: PulseExecutableWorld<Installed> = PulseExecutableWorld::install(canonical)
        .unwrap_or_else(|failure| panic!("install hostile replacement world: {failure}"));
    let binary = CargoBuiltPlatformPulse::exact()
        .unwrap_or_else(|failure| panic!("resolve hostile Cargo executable: {failure}"));
    let awaiting: PulseExecutableWorld<AwaitingFirstFrame> = installed
        .launch(binary)
        .unwrap_or_else(|failure| panic!("launch hostile replacement process: {failure}"));
    let first_frame_deadline = PulseCausalActionManifest::checked_in()
        .expect("checked-in causal deadlines are valid")
        .first_frame_deadline();
    let published: PulseExecutableWorld<Published<InitialBlue>> = awaiting
        .await_first_frame(Instant::now() + first_frame_deadline)
        .unwrap_or_else(|failure| panic!("reach hostile initial frame: {failure}"));
    let second_current = reach_second_current(published);
    let awaiting_green: PulseExecutableWorld<AwaitingReplacement> = second_current
        .apply_green(green_delta)
        .unwrap_or_else(|failure| panic!("apply hostile green action: {failure}"));
    let expired = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .expect("representable expired replacement deadline");
    let failure = match awaiting_green.await_green_successor(expired) {
        Ok(_) => panic!("expired green observation must fail"),
        Err(failure) => failure,
    };

    assert!(matches!(
        failure.primary(),
        PulseExecutableWorldFailure::WatchedObservation(WatchedPulseObservationFailure::Deadline(
            WatchedPulseTransition::GreenReplacement
        ))
    ));
    assert_eq!(failure.teardown().forced_process_termination(), Some(true));
    assert_eq!(failure.teardown().lifecycle_reader_joined(), Some(true));
    assert!(failure.teardown().discarded_lifecycle_envelopes().is_some());
    assert_eq!(failure.teardown().installation_removed(), Some(true));
    assert!(failure.teardown().all_owned_resources_released());
    discard_expected_failure_artifact(failure);
}

fn reach_second_current(
    published: PulseExecutableWorld<Published<InitialBlue>>,
) -> PulseExecutableWorld<Published<SecondCurrent>> {
    let first = published
        .reach_native_input(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile native-input stage: {failure}"))
        .publish_first_query_value(QueryStatusV1)
        .unwrap_or_else(|failure| panic!("publish hostile first Query value: {failure}"))
        .await_first_query_value(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile first Query value: {failure}"));
    let snapshot: PulseExecutableWorld<Published<SnapshotCaptured<FirstCurrent>>> = first
        .await_visual_snapshot(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile snapshot stage: {failure}"));
    let traced: PulseExecutableWorld<Published<IdentityTraced<FirstCurrent>>> = snapshot
        .await_identity_trace(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile trace stage: {failure}"));
    let overlay: PulseExecutableWorld<Published<OverlayPublished<FirstCurrent>>> = traced
        .await_overlay_published(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile overlay stage: {failure}"));
    let cleared: PulseExecutableWorld<Published<OverlayCleared<FirstCurrent>>> = overlay
        .await_overlay_cleared(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile clear stage: {failure}"));
    cleared
        .publish_second_query_value(QueryStatusV2)
        .unwrap_or_else(|failure| panic!("publish hostile second Query value: {failure}"))
        .await_second_query_value(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| panic!("reach hostile second Query value: {failure}"))
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
