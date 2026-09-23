use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::thread;
use std::time::{Duration, Instant};

use crate::native_platform::{
    CertifiedNativePlatform, NativePlatformContract, NativePlatformFailure,
};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

mod evidence;

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn certified_native_boundary_world_retains_click_time_pointer_after_cursor_moves() {
    let platform = CertifiedNativePlatform::certified().expect("native observation is qualified");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(CargoBuiltPlatformPulse::launch_native_phase6)
        .expect("the native phase 6 product process launches");
    let process_id = launch.process.id();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute_boundary_world(&platform, &mut launch, process_id)
    }));
    if let Err(failure) = outcome {
        finalize_failed_world(&platform, &mut launch.process, process_id);
        resume_unwind(failure);
    }
}

fn execute_boundary_world(
    platform: &CertifiedNativePlatform,
    launch: &mut crate::product_process::NativePhase2ProcessLaunch,
    process_id: u32,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    let client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("one process-owned native client area appears");
    let capture = platform
        .capture_client_area(&client)
        .expect("the native client area is externally capturable");
    let point = crate::external_observation::NativeClientPixelPoint::interior(
        &capture,
        capture.width() / 3,
        capture.height() / 2,
        2,
    )
    .expect("the click point is inside the captured client area");
    let delivered = match platform.deliver_pointer_activation(&client, point) {
        Ok(delivered) => delivered,
        Err(NativePlatformFailure::InputEnvironment(denial)) => {
            eprintln!(
                "native input environment denied: {}",
                serde_json::json!({
                    "schema": "worth-ui-native-phase6-environment-denial-v1",
                    "requirement": crate::native_platform::world_requirement(6),
                    "delivery_route": "system-input-to-native-message-queue",
                    "result": "environment-denied",
                    "denial": denial.to_string(),
                })
            );
            panic!("the Windows input world is not qualified: {denial}");
        }
        Err(failure) => panic!("the real OS input boundary failed: {failure}"),
    };
    let click_screen = delivered.qualified_screen_point();
    thread::sleep(Duration::from_millis(250));
    platform
        .deliver_wheel_deltas(&client)
        .expect("the focused process window receives vertical and horizontal wheel input");
    thread::sleep(Duration::from_millis(250));

    let bounds = platform
        .observe_bound_client_area(&client)
        .expect("the bound client area remains stable after delivery")
        .bounds();
    let moved_screen = (bounds.right() - 3, bounds.bottom() - 3);
    assert_ne!(moved_screen, click_screen);
    platform
        .move_cursor(moved_screen)
        .expect("the cursor moves after the click has been delivered");
    thread::sleep(Duration::from_millis(250));

    let close = platform
        .request_normal_close(&client)
        .expect("the real OS window accepts one normal close");
    assert_eq!(close.process_id(), process_id);
    assert_eq!(close.request_count(), 1);
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("native product exits successfully after OS close");
    assert!(exit.status().success());
    platform
        .verify_process_window_released(process_id)
        .expect("the process leaves no native window");

    let mut stdout = String::new();
    launch
        .stdout
        .read_to_string(&mut stdout)
        .expect("native phase 6 report is readable");
    let evidence: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("versioned native phase 6 evidence is JSON");
    evidence::assert_phase6_evidence(
        &evidence,
        evidence::NativePhase6WorldFacts {
            capture: &capture,
            bounds,
            click_screen,
            moved_screen,
        },
    );
    let pointer_witnesses = u64::from(evidence["input"]["last_pointer_button"].is_object());
    assert_eq!(
        pointer_witnesses, 1,
        "phase 6 boundary world must retain one event-time pointer witness"
    );
}

fn finalize_failed_world(
    platform: &CertifiedNativePlatform,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    process_id: u32,
) {
    let teardown = process.terminate_after_failure(Instant::now() + Duration::from_secs(5));
    let window_release = platform.verify_process_window_released(process_id);
    eprintln!(
        "native phase 6 world failure: {}",
        serde_json::json!({
            "schema": "worth-ui-native-phase6-world-failure-v1",
            "process_id": process_id,
            "teardown": format!("{teardown:?}"),
            "window_release": format!("{window_release:?}"),
        })
    );
    assert!(
        teardown.is_ok(),
        "failed world teardown was incomplete: {teardown:?}"
    );
    assert!(
        window_release.is_ok(),
        "failed world retained a process window: {window_release:?}"
    );
}
