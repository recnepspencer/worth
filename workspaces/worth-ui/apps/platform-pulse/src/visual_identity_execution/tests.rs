use super::*;

#[test]
fn capture_failure_cannot_strand_visual_state_as_transitioning() {
    let mut execution = PlatformPulseVisualIdentityExecution::new();
    execution.state = Some(PlatformPulseVisualIdentityState::Transitioning);

    assert!(execution
        .install_advance_result(Err(PlatformPulseVisualExecutionDenial::SnapshotDeadline(
            PlatformPulseVisualCapturePhase::Initial,
        )))
        .is_err());
    assert!(matches!(
        execution.state,
        Some(PlatformPulseVisualIdentityState::Failed)
    ));
}

#[test]
fn first_frame_capture_uses_typed_readiness_without_a_prerequisite_sleep() {
    let now = Instant::now();
    let mut execution = PlatformPulseVisualIdentityExecution::new();
    // Capture only arms for the visual identity journey, which `new` reads
    // from the process environment; the test states it instead.
    execution.enabled = true;

    execution
        .arm_after_first_frame(now)
        .expect("first frame arms visual capture");

    let Some(PlatformPulseVisualIdentityState::Settling { begin_at, deadline }) = execution.state
    else {
        panic!("first frame must enter typed mounted-frame readiness")
    };
    assert_eq!(begin_at, now);
    assert_eq!(deadline, now + REPLACEMENT_FRAME_DEADLINE);
}
