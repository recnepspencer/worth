//! One real Pulse offscreen run crosses the production watcher/continuation/presentation path.
use std::{cell::RefCell, rc::Rc, time::Duration};
use worth_ui_native_platform::{UiNativeOffscreenStart, UiNativeOffscreenSettle, UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform};
use worth_ui_platform_pulse::{observation_contract::PlatformPulseQueryProjectionPosture, visual_identity_pulse::PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT};

#[test]
fn real_pulse_source_and_action_stops_resume_original_request_before_close() {
    // This is the existing offscreen fixture's host-turn ceiling, not an operation retry quota.
    const HOST_TURNS: u32 = 10_000;
    let probe = Rc::new(RefCell::new(super::super::QueryContinuationForTest::default()));
    let launch = crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration::from_arguments(std::iter::empty()).unwrap();
    let publisher = crate::lifecycle_observation_publication::PlatformPulseObservationPublisher::start().unwrap();
    let mut application = super::PlatformPulseApplication::offscreen(launch, publisher);
    application.query_probe = Some(Rc::clone(&probe));
    let extent = PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT;
    let platform = WorthUiNativePlatform::prepare(UiNativePlatformProfile::single_window(UiNativeWindowSpec::new("Pulse retained Query continuation", extent))).unwrap();
    let mut session = match platform.start_offscreen(application, extent, 1.0) {
        UiNativeOffscreenStart::Started(session) => session,
        UiNativeOffscreenStart::Ended(outcome) => panic!("real Pulse preparation failed: {outcome:?}"),
    };
    let mut settled = session.run_until_idle(HOST_TURNS);
    assert!(!matches!(settled, UiNativeOffscreenSettle::Busy));
    let stopped_attempts = probe.borrow().cancelled_stops;
    if stopped_attempts == 0 {
        assert!(session.await_wake(Duration::from_secs(2)), "actual Query watcher must deliver its checked-in source");
        settled = session.run_until_idle(HOST_TURNS);
        assert!(!matches!(settled, UiNativeOffscreenSettle::Busy));
    }
    assert!(probe.borrow().cancelled_stops >= 1, "real source ingress must yield instead of stopping the product");
    // A later ordinary viewport event supplies fresh request authority. No self-wake pump.
    if !matches!(settled, UiNativeOffscreenSettle::Exited { .. }) {
        session.resize([extent[0] - 16, extent[1]]);
        settled = session.run_until_idle(HOST_TURNS);
    }
    if matches!(settled, UiNativeOffscreenSettle::Idle { .. }) && session.await_wake(Duration::from_secs(2)) {
        settled = session.run_until_idle(HOST_TURNS);
    }
    assert!(matches!(settled, UiNativeOffscreenSettle::Exited { .. }), "the actual retained-work completion must return Close, not strand the driver in Continue: {settled:?}");
    {
        let observed = probe.borrow();
        assert_eq!(observed.cancelled_stops, 2, "both actual source and action attempts stop on cancelled authentication");
        assert_eq!(observed.attempts, 4, "each original attempt has exactly one fresh continuation");
        assert_eq!(observed.revisions, [0, 1, 1], "source revision is preserved through the one action effect");
        assert_eq!(observed.basis_versions.len(), 3);
        assert_eq!(observed.basis_versions[1], observed.basis_versions[0] + 1, "one genuine source commit, no reconstructed command");
        assert_eq!(observed.basis_versions[2], observed.basis_versions[1] + 1, "one genuine action commit from the original retained command");
        let current = observed.issued.iter().filter(|evidence| evidence.posture() == PlatformPulseQueryProjectionPosture::Current).collect::<Vec<_>>();
        assert_eq!(current.len(), 2);
        assert_eq!(current[0].native_value(), Some("Scheduled"), "independent checked-in source expectation");
        assert_eq!(current[1].native_value(), Some("Deployed"), "real product action consequence");
        assert!(observed.stopped_request.is_some());
        assert_eq!(observed.completed_request, observed.stopped_request, "the same actual UiIntent request completes");
        let pending = observed.pending_census.expect("actual pending product port census");
        assert_eq!((pending.submitted(), pending.received(), pending.settled(), pending.retained()), (1, 1, 0, 1));
        let completed = observed.completed_census.expect("actual completed product port census");
        assert_eq!((completed.submitted(), completed.received(), completed.settled(), completed.retained()), (1, 1, 1, 0));
        assert_eq!(observed.intent_outcomes, 1, "one complete actual Intent causal trace reaches presentation");
        assert_eq!(observed.presented, observed.issued, "each issued fact reaches one actual mounted publication");
        assert!(observed.close_refused_before_presentation, "real native close gate retains Query and shell while presentation is owed");
        assert!(observed.close_waited_for_presentation, "actual external-close contract must let retained presentation progress instead of stranding it");
    }
    match session.close(HOST_TURNS) {
        UiNativePlatformOutcome::Closed(receipt) => assert!(receipt.terminal_census().is_zero()),
        outcome => panic!("actual driver closes only after original publication is admitted: {outcome:?}"),
    }
}
