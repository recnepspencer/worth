use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::native_platform::{
    CertifiedGraphicsRecord, CertifiedNativePlatform, NativePlatformContract, NativePlatformFailure,
};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

#[path = "native_phase2/resource_evidence.rs"]
mod resource_evidence;

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn certified_native_boundary_world_presents_quiesces_and_closes_without_residue() {
    let platform = CertifiedNativePlatform::certified().expect("native observation is qualified");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(CargoBuiltPlatformPulse::launch_native_phase2)
        .expect("the one product binary launches under the native desktop lease");
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
    let mut client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("one process-owned native client area appears");
    let capture = await_exact_pixels(platform, &mut client, &mut launch.process, deadline);
    assert_eq!(capture.process_id(), process_id);
    assert_eq!(capture.capture_count(), 1);
    assert_exact_control_points(&capture);
    std::thread::sleep(Duration::from_millis(250));
    let unchanged = platform
        .capture_client_area(&client)
        .expect("quiescent client area remains externally observable");
    assert_quiescent_control_points(&capture, &unchanged);
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
        .expect("native phase report is readable");
    let evidence: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("versioned native evidence is JSON");
    assert_exact_native_evidence(&evidence, &capture);
}

fn finalize_failed_world(
    platform: &CertifiedNativePlatform,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    process_id: u32,
) {
    let teardown = process.terminate_after_failure(Instant::now() + Duration::from_secs(5));
    let window_release = platform.verify_process_window_released(process_id);
    eprintln!(
        "native world failure: {}",
        serde_json::json!({
            "schema": "worth-ui-native-world-failure-v1",
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

fn assert_exact_native_evidence(
    evidence: &serde_json::Value,
    capture: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase2-evidence-v1");
    assert_exact_presentation(&evidence["presentation"], capture);
    assert_exact_attribution(&evidence["presentation"], &evidence["runtime_attribution"]);
    assert_exact_counters(&evidence["counters"]);
    assert_exact_graphics(&evidence["graphics"]);
    resource_evidence::assert_exact_resource_evidence(
        evidence,
        CertifiedNativePlatform::CREATION_SURFACE_SUCCESSION,
    );
}

fn assert_quiescent_control_points(
    initial: &crate::external_observation::NativeClientPixelCapture,
    unchanged: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(unchanged.process_id(), initial.process_id());
    assert_eq!(
        [unchanged.width(), unchanged.height()],
        [initial.width(), initial.height()]
    );
    let points = [
        (initial.width() / 4, initial.height() / 4),
        (initial.width() / 2, initial.height() / 2),
        (initial.width() * 3 / 4, initial.height() * 3 / 4),
    ];
    for (x, y) in points {
        assert_eq!(pixel(unchanged, x, y), pixel(initial, x, y));
        assert_eq!(pixel(unchanged, x, y), [47, 129, 247, 255]);
    }
}

fn assert_exact_presentation(
    presentation: &serde_json::Value,
    capture: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(
        presentation["presented_source"],
        serde_json::json!([47, 129, 247, 255])
    );
    assert_eq!(
        presentation["retained_center"],
        presentation["presented_source"]
    );
    assert_eq!(
        presentation["retained_baseline"],
        serde_json::json!([0, 0, 0, 0])
    );
    assert_eq!(
        presentation["client_physical_size"],
        serde_json::json!([capture.width(), capture.height()])
    );
    assert_eq!(
        presentation["logical_bounds_milli"],
        serde_json::json!([16_000, 12_000, 128_000, 72_000])
    );
}

fn assert_exact_attribution(
    presentation: &serde_json::Value,
    runtime_attribution: &serde_json::Value,
) {
    for identity in [
        "frame",
        "surface",
        "binding",
        "mounted_instance",
        "node_receipt",
        "presentation_attempt",
    ] {
        assert!(runtime_attribution[identity]
            .as_u64()
            .is_some_and(|value| value > 0));
        assert_eq!(presentation[identity], runtime_attribution[identity]);
    }
    assert_eq!(
        runtime_attribution["authored_provenance_digest"],
        expected_native_seed_authored_provenance_digest(),
        "native presentation must retain the authored seed declaration"
    );
    assert_eq!(
        runtime_attribution["authored_semantic_identity_digest"],
        expected_native_seed_authored_semantic_identity_digest(),
        "native presentation must retain the authored seed identity"
    );
}

fn assert_exact_counters(counters: &serde_json::Value) {
    for counter in [
        "surface_acquisitions",
        "queue_submissions",
        "presents",
        "readiness_signals",
        "redraw_turns",
        "idle_wait_turns",
    ] {
        assert_eq!(counters[counter], 1, "unexpected {counter}");
    }
    assert_eq!(counters["render_passes"], 2);
    assert_eq!(counters["port_crossings"], 4);
    assert!(counters["coalesced_wakes"]
        .as_u64()
        .is_some_and(|count| count <= 4));
}

/// The graphics axes are the certified record's, not a vendor's spelling: the
/// same courtroom certifies DX12 on Windows and software Vulkan under Xvfb.
fn assert_exact_graphics(graphics: &serde_json::Value) {
    let record = CertifiedGraphicsRecord::from_certified_manifest();
    assert_eq!(graphics["backend"], record.backend());
    assert!(
        graphics["device_type"]
            .as_str()
            .is_some_and(|device_type| record.admits_device_type(device_type)),
        "the record does not admit {}",
        graphics["device_type"]
    );
    assert_eq!(graphics["surface_format"], record.surface_format());
    assert_eq!(graphics["present_mode"], record.present_mode());
    assert_eq!(graphics["alpha_mode"], record.composite_alpha());
    assert_eq!(graphics["retained_format"], record.target_format());
    assert_eq!(graphics["event_loop_thread_matches_launch"], true);
    assert_eq!(
        graphics["event_loop_thread_posture"],
        "main-thread-required"
    );
    assert!(graphics["max_texture_dimension_2d"]
        .as_u64()
        .is_some_and(|limit| limit >= 16_384));
    assert!(graphics["event_loop_thread"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
}

fn expected_native_seed_authored_provenance_digest() -> u64 {
    independent_text_digest("app/native_seed.wui") ^ 1_u64.rotate_left(13)
}

fn expected_native_seed_authored_semantic_identity_digest() -> u64 {
    independent_text_digest("component:platform.pulse.native_seed.rectangle")
}

fn independent_text_digest(text: &str) -> u64 {
    text.as_bytes()
        .iter()
        .fold(0xCBF2_9CE4_8422_2325_u64, |digest, byte| {
            digest.wrapping_mul(0x0000_0100_0000_01B3) ^ u64::from(*byte)
        })
}

fn await_exact_pixels(
    platform: &CertifiedNativePlatform,
    client: &mut <CertifiedNativePlatform as NativePlatformContract>::BoundClientArea,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    deadline: Instant,
) -> crate::external_observation::NativeClientPixelCapture {
    let process_id = process.id();
    let mut last_center = None;
    let mut last_error = None;
    loop {
        match platform.capture_client_area(client) {
            Ok(capture) => {
                last_center = Some(center(&capture));
                if last_center == Some([47, 129, 247, 255]) {
                    return capture;
                }
            }
            Err(NativePlatformFailure::BoundClientAreaChanged) => {
                last_error = Some(NativePlatformFailure::BoundClientAreaChanged);
                *client = platform
                    .bind_process_client_area(process_id, deadline)
                    .expect("changed client area rebinds to one stable process window");
            }
            Err(error) => last_error = Some(error),
        }
        let status = process
            .observed_exit()
            .expect("native process can be polled");
        assert!(
            status.is_none(),
            "native process exited before pixel observation: {status:?}; center={last_center:?}; error={last_error:?}"
        );
        assert!(
            Instant::now() < deadline,
            "qualified client pixels did not appear; center={last_center:?}; error={last_error:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn assert_exact_control_points(capture: &crate::external_observation::NativeClientPixelCapture) {
    let points = [
        (capture.width() / 4, capture.height() / 4),
        (capture.width() / 2, capture.height() / 2),
        (capture.width() * 3 / 4, capture.height() * 3 / 4),
    ];
    for (x, y) in points {
        assert_eq!(pixel(capture, x, y), [47, 129, 247, 255]);
    }
}

fn center(capture: &crate::external_observation::NativeClientPixelCapture) -> [u8; 4] {
    pixel(capture, capture.width() / 2, capture.height() / 2)
}

fn pixel(
    capture: &crate::external_observation::NativeClientPixelCapture,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let index = ((y * capture.width() + x) * 4) as usize;
    capture.rgba()[index..index + 4]
        .try_into()
        .expect("one RGBA control point")
}
