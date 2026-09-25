use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseQueryProjectionPosture,
    PlatformPulseSemanticFocusCause,
};

use crate::adjudication::dashboard_visual_oracle as dashboard;
use crate::external_observation::{NativeClientPixelPoint, NativeKeyboardCommand};
use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};
use crate::native_platform::{CertifiedNativePlatform, NativePlatformContract};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};
use crate::source_delta::{QueryStatusV1, QueryStatusV2};

#[path = "query_application_launch_pixels.rs"]
mod pixels;
use pixels::{changed_region_pixels, changed_status_pixels};

#[test]
fn authored_query_revisions_reach_the_real_pulse_process() {
    let mut installation = IsolatedPulseInstallation::install(CanonicalPlatformPulse::checked_in())
        .expect("install the current complete Pulse source");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .expect("resolve the product executable")
        .launch(
            installation.source_root(),
            installation.source_root(),
            installation.source_root(),
        )
        .expect("launch the product with its authored sources");
    let deadline = Instant::now() + Duration::from_secs(35);
    let mut first_frame = false;
    let mut pending_issued = false;
    let mut pending_published = false;
    while !first_frame || !pending_issued || !pending_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("product lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::FirstFramePublished(frame) => {
                assert!(frame.actual_native_effect_count() > 0);
                first_frame = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(projection.projection_identity(), "platform.pulse.status");
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Pending
                );
                assert_eq!(projection.native_value(), None);
                pending_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().projection_identity(),
                    "platform.pulse.status"
                );
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Pending
                );
                pending_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before its first authored Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let platform = CertifiedNativePlatform::certified().expect("native desktop available");
    let client = platform
        .bind_process_client_area(launch.process.id(), deadline)
        .expect("product presents one native client area");
    let pending_pixels = platform
        .capture_client_area(&client)
        .expect("capture pending status");
    QueryStatusV1
        .apply(&installation)
        .expect("publish the first external status revision");
    let mut current_issued = false;
    let mut current_published = false;
    while !current_issued || !current_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("current Query lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(projection.native_value(), Some(QueryStatusV1::VALUE));
                current_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(
                    publication.projection().native_value(),
                    Some(QueryStatusV1::VALUE)
                );
                current_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the first current Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let online_pixels = platform
        .capture_client_area(&client)
        .expect("capture online status");
    assert!(changed_status_pixels(&pending_pixels, &online_pixels) >= 9);
    QueryStatusV2
        .apply(&installation)
        .expect("publish the second external status revision");
    let mut second_issued = false;
    let mut second_published = false;
    while !second_issued || !second_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("second current Query lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(projection.native_value(), Some(QueryStatusV2::VALUE));
                second_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(
                    publication.projection().native_value(),
                    Some(QueryStatusV2::VALUE)
                );
                second_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the second current Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let synchronized_pixels = platform
        .capture_client_area(&client)
        .expect("capture synchronized status");
    assert!(changed_status_pixels(&online_pixels, &synchronized_pixels) >= 9);
    let baseline = platform
        .capture_client_area(&client)
        .expect("capture current dashboard");
    let bell_target = NativeClientPixelPoint::interior(
        &baseline,
        1420 * baseline.width() / 1536,
        32 * baseline.height() / 1024,
        1,
    )
    .expect("signals target lies inside the native client area");
    platform
        .deliver_pointer_activation(&client, bell_target)
        .expect("open the signals popover through the native target");
    let mut signals_focused = false;
    while !signals_focused {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("signals lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus) => {
                signals_focused = focus.cause() == PlatformPulseSemanticFocusCause::PortalInitial;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed while opening signals: {failure:?}")
            }
            _ => {}
        }
    }
    let signals_region = [1207, 61, 307, 253];
    let signals_open = loop {
        let capture = platform
            .capture_client_area(&client)
            .expect("capture signals popover");
        if changed_region_pixels(&baseline, &capture, signals_region) >= 24 {
            break capture;
        }
        assert!(
            Instant::now() < deadline,
            "signals popover did not reach native pixels"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    platform
        .deliver_keyboard_command(&client, NativeKeyboardCommand::Escape)
        .expect("dismiss signals through native Escape");
    let mut signals_dismissed = false;
    let mut focus_restored = false;
    while !signals_dismissed || !focus_restored {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("signals dismissal event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::PortalDismissed(_) => signals_dismissed = true,
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus) => {
                focus_restored |=
                    focus.cause() == PlatformPulseSemanticFocusCause::PortalRestoration;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed while dismissing signals: {failure:?}")
            }
            _ => {}
        }
    }
    let signals_closed = loop {
        let capture = platform
            .capture_client_area(&client)
            .expect("capture restored dashboard");
        if changed_region_pixels(&signals_open, &capture, signals_region) >= 24 {
            break capture;
        }
        assert!(
            Instant::now() < deadline,
            "Escape did not restore native pixels"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let [badge_x, badge_y, badge_width, badge_height] = dashboard::QUERY_POSTURE_REGION;
    let review_target = NativeClientPixelPoint::interior(
        &signals_closed,
        (badge_x + badge_width / 2) * signals_closed.width() / 1536,
        (badge_y + badge_height / 2) * signals_closed.height() / 1024,
        1,
    )
    .expect("review target lies inside the native client area");
    platform
        .deliver_pointer_activation(&client, review_target)
        .expect("open the review modal through the current product target");
    let mut review_presented = false;
    let mut review_focus = None;
    while !review_presented {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("review modal lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus) => {
                review_presented = focus.cause() == PlatformPulseSemanticFocusCause::PortalInitial;
                if review_presented {
                    review_focus = Some(format!("{focus:?}"));
                }
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed while opening review: {failure:?}")
            }
            _ => {}
        }
    }
    let visible_modal = loop {
        let capture = platform
            .capture_client_area(&client)
            .expect("capture presented review modal");
        if changed_region_pixels(&signals_closed, &capture, [500, 250, 550, 520]) >= 24 {
            break capture;
        }
        if Instant::now() >= deadline {
            let mut later = Vec::new();
            for _ in 0..16 {
                let Ok(envelope) = launch
                    .lifecycle
                    .next(Instant::now() + Duration::from_millis(10))
                else {
                    break;
                };
                later.push(format!("{:?}", envelope.outcome()));
            }
            panic!(
                "review modal did not reach native pixels; focus={review_focus:?}; capture={}x{}; changed={}; full_changed={}; later={later:?}; trace={:?}",
                capture.width(),
                capture.height(),
                changed_region_pixels(&signals_closed, &capture, [500, 250, 550, 520]),
                changed_region_pixels(&signals_closed, &capture, [0, 0, 1536, 1024]),
                launch.lifecycle.failure_snapshot()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut previous = visible_modal;
    let mut stable_captures = 0;
    let modal = loop {
        std::thread::sleep(Duration::from_millis(30));
        let capture = platform
            .capture_client_area(&client)
            .expect("capture stable review action");
        stable_captures = if changed_region_pixels(&previous, &capture, [780, 625, 245, 125]) < 12 {
            stable_captures + 1
        } else {
            0
        };
        if stable_captures == 3 {
            break capture;
        }
        assert!(
            Instant::now() < deadline,
            "review action pixels did not settle"
        );
        previous = capture;
    };
    let approve_target = NativeClientPixelPoint::interior(
        &modal,
        892 * modal.width() / 1536,
        681 * modal.height() / 1024,
        1,
    )
    .expect("approve target lies inside the native client area");
    platform
        .deliver_pointer_activation(&client, approve_target)
        .expect("approve the review through the native modal target");
    let mut action_issued = false;
    let mut action_published = false;
    let mut action_observations = Vec::new();
    while !action_issued || !action_published {
        let envelope = launch.lifecycle.next(deadline).unwrap_or_else(|denial| {
            let after = platform.capture_client_area(&client).ok();
            panic!(
                "action Query lifecycle event: {denial:?}; recent={:?}; modal_changed={:?}; trace={:?}",
                action_observations,
                after.as_ref().map(|after| changed_region_pixels(&modal, after, [500, 250, 550, 520])),
                launch.lifecycle.failure_snapshot()
            )
        });
        action_observations.push(format!("{:?}", envelope.outcome()));
        if action_observations.len() > 12 {
            action_observations.remove(0);
        }
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(projection.native_value(), Some("Deployed"));
                action_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(publication.projection().native_value(), Some("Deployed"));
                action_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the authored action consequence: {failure:?}")
            }
            _ => {}
        }
    }
    let deployed_pixels = platform
        .capture_client_area(&client)
        .expect("capture the accepted action consequence");
    assert!(changed_status_pixels(&synchronized_pixels, &deployed_pixels) >= 9);
    platform
        .request_normal_close(&client)
        .expect("close product window");
    let mut shutdown = false;
    while !shutdown {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("shutdown lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::ShutdownCompleted(_) => shutdown = true,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during normal close: {failure:?}")
            }
            _ => {}
        }
    }
    SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("product exits after normal close");
    platform
        .verify_process_window_released(launch.process.id())
        .expect("native window released");
    installation
        .close()
        .expect("release the isolated source installation");
}
