use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::native_platform::{NativePlatformContract, WindowsNativePlatform};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

const CLIENT_GROWTH: [u32; 2] = [64, 40];
const INITIAL_RGBA: [u8; 4] = [47, 129, 247, 255];
const POST_RESTORE_RGBA: [u8; 4] = [63, 185, 80, 255];

#[test]
#[ignore = "requires the serialized interactive Windows 11 DX12 desktop"]
fn windows_native_boundary_world_actuates_resize_minimize_restore_and_reconstruction() {
    let platform = WindowsNativePlatform::certified().expect("Windows observation is qualified");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(CargoBuiltPlatformPulse::launch_native_phase8)
        .expect("the native Phase 8 product process launches");
    let process_id = launch.process.id();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute(&platform, &mut launch, process_id)
    }));
    if let Err(failure) = outcome {
        finalize_failed_world(&platform, &mut launch.process, process_id);
        resume_unwind(failure);
    }
}

fn execute(
    platform: &WindowsNativePlatform,
    launch: &mut crate::product_process::NativePhase2ProcessLaunch,
    process_id: u32,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("one process-owned native client area appears");
    let initial = platform
        .observe_bound_client_area(&client)
        .expect("the initial client area is stable");
    await_presented_pixels(
        platform,
        &mut client,
        &mut launch.process,
        deadline,
        None,
        INITIAL_RGBA,
    );
    let resized_client = growing_client_extent(initial.bounds());
    let resized = platform
        .resize_bound_client_area(&mut client, resized_client, deadline)
        .expect("the OS applies the requested client resize");
    assert_ne!(initial.bounds(), resized.bounds());
    assert_eq!(
        [resized.bounds().width(), resized.bounds().height()],
        resized_client
    );
    await_presented_pixels(
        platform,
        &mut client,
        &mut launch.process,
        deadline,
        Some((initial, resized)),
        INITIAL_RGBA,
    );

    let visibility = platform
        .minimize_and_restore_bound_client_area(&mut client, deadline)
        .expect("the real window reaches minimized and restored states");
    assert_eq!(visibility.minimized_observations(), 1);
    assert_eq!(visibility.restored_observations(), 1);
    assert_eq!(visibility.restored_client().bounds(), resized.bounds());
    await_presented_pixels(
        platform,
        &mut client,
        &mut launch.process,
        deadline,
        Some((initial, resized)),
        POST_RESTORE_RGBA,
    );
    let external = platform
        .capture_client_area(&client)
        .expect("the restored resized client area is compositor-visible");
    assert_eq!([external.width(), external.height()], resized_client);

    platform
        .request_normal_close(&client)
        .expect("the restored window accepts normal close");
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("the Phase 8 process exits successfully");
    assert!(exit.status().success());
    platform
        .verify_process_window_released(process_id)
        .expect("the process leaves no native window");

    let mut stdout = String::new();
    launch
        .stdout
        .read_to_string(&mut stdout)
        .expect("Phase 8 evidence is readable");
    let evidence: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Phase 8 evidence is JSON");
    assert_evidence(&evidence, initial, resized, resized_client, &external);
}

fn await_presented_pixels(
    platform: &WindowsNativePlatform,
    client: &mut <WindowsNativePlatform as NativePlatformContract>::BoundClientArea,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    deadline: Instant,
    resized: Option<(
        crate::external_observation::ProcessBoundNativeClientAreaObservation,
        crate::external_observation::ProcessBoundNativeClientAreaObservation,
    )>,
    expected_rgba: [u8; 4],
) {
    let process_id = process.id();
    let mut last_capture = None;
    loop {
        match platform.capture_client_area(client) {
            Ok(capture) if presented_pixels_are_current(&capture, resized, expected_rgba) => return,
            Err(crate::native_platform::NativePlatformFailure::BoundClientAreaChanged) => {
                *client = platform
                    .bind_process_client_area(process_id, deadline)
                    .expect("changed Phase 8 client area rebinds to the same process window");
            }
            Ok(capture) => {
                last_capture = Some((
                    [capture.width(), capture.height()],
                    pixel(&capture, [capture.width() / 2, capture.height() / 2]),
                ));
            }
            Err(_) => {}
        }
        assert!(
            process.observed_exit().unwrap().is_none(),
            "Phase 8 process exited before its product pixels became current"
        );
        assert!(
            Instant::now() < deadline,
            "Phase 8 product pixels did not become {expected_rgba:?} before the deadline; last={last_capture:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn presented_pixels_are_current(
    capture: &crate::external_observation::NativeClientPixelCapture,
    resized: Option<(
        crate::external_observation::ProcessBoundNativeClientAreaObservation,
        crate::external_observation::ProcessBoundNativeClientAreaObservation,
    )>,
    expected_rgba: [u8; 4],
) -> bool {
    if pixel(capture, [capture.width() / 2, capture.height() / 2]) != expected_rgba {
        return false;
    }
    let Some((initial, resized)) = resized else {
        return true;
    };
    if [capture.width(), capture.height()] != [resized.bounds().width(), resized.bounds().height()]
    {
        return false;
    }
    let inset = (16_u32 * resized.dpi() + 48) / 96;
    let stale_right = initial.bounds().width().saturating_sub(inset);
    let successor_right = resized.bounds().width().saturating_sub(inset);
    let successor_only_x = stale_right + (successor_right - stale_right) / 2;
    pixel(capture, [successor_only_x, capture.height() / 2]) == expected_rgba
}

fn growing_client_extent(initial: crate::external_observation::NativeClientAreaBounds) -> [u32; 2] {
    [
        initial
            .width()
            .checked_add(CLIENT_GROWTH[0])
            .expect("qualified client width admits bounded growth"),
        initial
            .height()
            .checked_add(CLIENT_GROWTH[1])
            .expect("qualified client height admits bounded growth"),
    ]
}

fn assert_evidence(
    evidence: &serde_json::Value,
    initial: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    resized: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    resized_client: [u32; 2],
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase8-evidence-v1");
    assert_eq!(initial.window(), resized.window());
    assert_eq!(initial.dpi(), resized.dpi());
    assert_snapshot_basis(evidence, resized_client, external);
    assert_reconstruction(evidence);
    assert_resource_lifecycle(evidence);
    assert_resized_pixels(initial, resized, external);
}

fn assert_snapshot_basis(
    evidence: &serde_json::Value,
    resized_client: [u32; 2],
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(
        evidence["presentation"]["client_physical_size"],
        serde_json::json!(resized_client)
    );
    assert_eq!(
        evidence["snapshot"]["client_physical_dimensions"],
        serde_json::json!(resized_client)
    );
    assert_eq!(
        evidence["snapshot"]["pixel_dimensions"],
        serde_json::json!(resized_client)
    );
    assert_eq!(
        evidence["snapshot"]["pixel_byte_count"],
        u64::from(external.width()) * u64::from(external.height()) * 4
    );
    let logical_bits = evidence["snapshot"]["viewport_logical_dimension_bits"]
        .as_array()
        .expect("logical viewport dimensions");
    let scale_bits = evidence["snapshot"]["scale_bits"]
        .as_array()
        .expect("snapshot scale");
    for axis in 0..2 {
        let logical = f32::from_bits(
            logical_bits[axis]
                .as_u64()
                .expect("logical dimension bits fit u32") as u32,
        );
        let scale = f32::from_bits(scale_bits[axis].as_u64().expect("scale bits fit u32") as u32);
        assert_eq!((logical * scale).round() as u32, resized_client[axis]);
    }
}

fn assert_reconstruction(evidence: &serde_json::Value) {
    let kinds = evidence["retained_frames"]
        .as_array()
        .expect("retained frame array")
        .iter()
        .map(|frame| frame["kind"].as_str().expect("frame kind"))
        .collect::<Vec<_>>();
    assert_eq!(kinds, ["Initial", "Reconstruction", "Reconstruction"]);
    let frames = evidence["retained_frames"]
        .as_array()
        .expect("retained frame array");
    assert_ne!(frames[1]["frame"], frames[2]["frame"]);
    assert_ne!(
        frames[1]["presentation_attempt"],
        frames[2]["presentation_attempt"]
    );
    assert_eq!(
        evidence["post_restore_presentation"]["retained_center_rgba8"],
        serde_json::json!(POST_RESTORE_RGBA)
    );
    assert_eq!(evidence["graphics_generations"]["device"], 1);
    assert_eq!(evidence["graphics_generations"]["surface"], 1);
}

fn assert_resource_lifecycle(evidence: &serde_json::Value) {
    assert!(evidence["surface_suspension"]["count"].as_u64().unwrap() >= 1);
    assert_eq!(
        evidence["surface_suspension"]["targetless_count"],
        evidence["surface_suspension"]["count"]
    );
    assert_eq!(evidence["peak"]["surfaces"], 1);
    assert_eq!(evidence["peak"]["devices"], 1);
    assert_eq!(evidence["peak"]["queues"], 1);
    assert_eq!(evidence["peak"]["retained_targets"], 2);
    assert_eq!(evidence["query_close_complete"], true);
    assert_eq!(evidence["intent_resources_empty"], true);
    assert_eq!(evidence["terminal_zero"], true);
}

fn assert_resized_pixels(
    initial: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    resized: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    let center = [external.width() / 2, external.height() / 2];
    assert_eq!(pixel(external, center), POST_RESTORE_RGBA);
    let inset = (16_u32 * resized.dpi() + 48) / 96;
    let stale_right = initial.bounds().width().saturating_sub(inset);
    let successor_right = resized.bounds().width().saturating_sub(inset);
    assert!(successor_right > stale_right);
    let successor_only_x = stale_right + (successor_right - stale_right) / 2;
    assert_eq!(
        pixel(external, [successor_only_x, external.height() / 2]),
        POST_RESTORE_RGBA
    );
    assert_ne!(
        pixel(external, [successor_right + 1, external.height() / 2]),
        POST_RESTORE_RGBA
    );
}

fn pixel(
    capture: &crate::external_observation::NativeClientPixelCapture,
    point: [u32; 2],
) -> [u8; 4] {
    let offset = ((point[1] * capture.width() + point[0]) * 4) as usize;
    capture.rgba()[offset..offset + 4]
        .try_into()
        .expect("courtroom pixel is one RGBA texel")
}

fn finalize_failed_world(
    platform: &WindowsNativePlatform,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    process_id: u32,
) {
    let teardown = process.terminate_after_failure(Instant::now() + Duration::from_secs(5));
    let window_release = platform.verify_process_window_released(process_id);
    assert!(
        teardown.is_ok(),
        "failed world teardown was incomplete: {teardown:?}"
    );
    assert!(
        window_release.is_ok(),
        "failed world retained a process window: {window_release:?}"
    );
}
