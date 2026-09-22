use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::native_platform::{
    CertifiedNativePlatform, NativePlatformContract, NativePlatformFailure,
};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn maximum_overlap_deltas_cross_public_runtime_native_pixels_and_exact_costs() {
    let platform = CertifiedNativePlatform::certified().expect("native observation is qualified");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(CargoBuiltPlatformPulse::launch_native_phase3)
        .expect("the product binary launches the fixed Phase 3 native world");
    let process_id = launch.process.id();
    let deadline = Instant::now() + Duration::from_secs(60);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute_phase3_world(&platform, &mut launch, process_id, deadline)
    }));
    if let Err(failure) = outcome {
        let teardown = launch
            .process
            .terminate_after_failure(Instant::now() + Duration::from_secs(5));
        let release = platform.verify_process_window_released(process_id);
        eprintln!(
            "WORTH_UI_PHASE3_FAILURE={{\"teardown\":\"{teardown:?}\",\"release\":\"{release:?}\"}}"
        );
        assert!(teardown.is_ok() && release.is_ok());
        resume_unwind(failure);
    }
}

fn execute_phase3_world(
    platform: &CertifiedNativePlatform,
    launch: &mut crate::product_process::NativePhase2ProcessLaunch,
    process_id: u32,
    deadline: Instant,
) {
    let mut client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("one process-owned native client area appears");
    let capture = await_final_pixels(platform, &mut client, &mut launch.process, deadline);
    let close = platform
        .request_normal_close(&client)
        .expect("the real OS window accepts one normal close");
    assert_eq!(close.request_count(), 1);
    SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("Phase 3 native world exits cleanly");
    platform
        .verify_process_window_released(process_id)
        .expect("the process leaves no native window");
    let mut retained_output = String::new();
    launch.stdout.read_to_string(&mut retained_output).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(retained_output.trim()).unwrap();
    assert_phase3_evidence(&evidence, [capture.width(), capture.height()]);
    assert_eq!(capture.process_id(), process_id);
    for (x, y) in [
        (capture.width() / 4, capture.height() / 4),
        (capture.width() / 2, capture.height() / 2),
        (capture.width() * 3 / 4, capture.height() * 3 / 4),
    ] {
        assert_eq!(pixel(&capture, x, y), [47, 129, 247, 255]);
    }
    println!("WORTH_UI_PHASE3_NATIVE_OBSERVATION={evidence}");
}

fn await_final_pixels(
    platform: &CertifiedNativePlatform,
    client: &mut <CertifiedNativePlatform as NativePlatformContract>::BoundClientArea,
    process: &mut crate::product_process::LivePlatformPulseProcess,
    deadline: Instant,
) -> crate::external_observation::NativeClientPixelCapture {
    let process_id = process.id();
    loop {
        match platform.capture_client_area(client) {
            Ok(capture)
                if pixel(&capture, capture.width() / 2, capture.height() / 2)
                    == [47, 129, 247, 255] =>
            {
                return capture;
            }
            Err(NativePlatformFailure::BoundClientAreaChanged) => {
                *client = platform
                    .bind_process_client_area(process_id, deadline)
                    .expect("changed Phase 3 client area rebinds exactly");
            }
            Ok(_) | Err(_) => {}
        }
        assert!(
            process.observed_exit().unwrap().is_none(),
            "Phase 3 process exited before its final pixels"
        );
        assert!(Instant::now() < deadline, "Phase 3 final pixels timed out");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn pixel(
    capture: &crate::external_observation::NativeClientPixelCapture,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let offset = (usize::try_from(y).unwrap() * usize::try_from(capture.width()).unwrap()
        + usize::try_from(x).unwrap())
        * 4;
    capture.rgba()[offset..offset + 4].try_into().unwrap()
}

fn assert_phase3_evidence(evidence: &serde_json::Value, client_extent: [u32; 2]) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase3-evidence-v1");
    assert_eq!(evidence["terminal_zero"], true);
    let frames = evidence["frames"].as_array().expect("phase3 frame array");
    assert_eq!(frames.len(), 8);
    let kinds = frames
        .iter()
        .map(|frame| frame["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            "Initial",
            "Unchanged",
            "Delta",
            "Delta",
            "Delta",
            "Delta",
            "Delta",
            "Delta",
        ]
    );
    assert_frame_pixels(frames);
    assert_physical_presentations(frames, client_extent);
    assert_zero_work(&frames[1]);
    assert_delta_cost(&frames[2], 1, 2_047);
    assert_delta_cost(&frames[4], 1_024, 1_024);
    assert_delta_cost(&frames[6], 2_048, 0);
    assert_full_target_pixels(&frames[0], 2_048);
    assert_full_target_pixels(&frames[2], 2_047);
    assert_full_target_pixels(&frames[4], 1_024);
    assert_full_target_pixels(&frames[6], 0);
    assert_full_target_pixels(&frames[7], 2_048);
    assert_eq!(frames[6]["cost"]["rendered_pixels"], 0);
}

fn assert_frame_pixels(frames: &[serde_json::Value]) {
    let blue = serde_json::json!([47, 129, 247, 255]);
    let yellow = serde_json::json!([242, 204, 96, 255]);
    let transparent = serde_json::json!([0, 0, 0, 0]);
    let expected = [
        &blue,
        &blue,
        &yellow,
        &blue,
        &yellow,
        &blue,
        &transparent,
        &blue,
    ];
    for (frame, expected) in frames.iter().zip(expected) {
        assert_eq!(&frame["center"], expected);
        assert_eq!(&frame["baseline"], expected);
    }
}

fn assert_physical_presentations(frames: &[serde_json::Value], extent: [u32; 2]) {
    let observed_client_pixels = u64::from(extent[0]) * u64::from(extent[1]);
    assert!(observed_client_pixels > 0);
    for frame in frames {
        let presents = frame["cost"]["presents"].as_u64().unwrap();
        let presented = frame["cost"]["presented_pixels"].as_u64().unwrap();
        let cleared = frame["cost"]["cleared_pixels"].as_u64().unwrap();
        let rendered = frame["cost"]["rendered_pixels"].as_u64().unwrap();
        assert_eq!(presented, observed_client_pixels * presents);
        assert_eq!(frame["cost"]["surface_acquisitions"], presents);
        assert_eq!(frame["cost"]["queue_submissions"], presents);
        assert_eq!(
            frame["cost"]["render_passes"].as_u64().unwrap(),
            presents * 2
        );
        assert_eq!(
            frame["cost"]["gpu_writes"].as_u64().unwrap(),
            u64::from(cleared > 0 || rendered > 0)
        );
    }
    assert_eq!(frames[0]["cost"]["surface_copies"], 0);
    for frame in &frames[2..] {
        let presents = frame["cost"]["presents"].as_u64().unwrap();
        assert_eq!(frame["cost"]["surface_copies"], presents);
    }
}

fn assert_full_target_pixels(frame: &serde_json::Value, replayed: u64) {
    let physical = frame["cost"]["presented_pixels"].as_u64().unwrap();
    assert_eq!(frame["cost"]["cleared_pixels"], physical);
    assert_eq!(frame["cost"]["rendered_pixels"], physical * replayed);
}

fn assert_zero_work(frame: &serde_json::Value) {
    for key in [
        "delta_rows_carried",
        "draw_list_mutations",
        "order_mutations",
        "logical_damage_regions",
        "retained_command_scans",
        "queue_submissions",
        "presents",
    ] {
        assert_eq!(frame["cost"][key], 0, "unexpected unchanged {key}");
    }
}

fn assert_delta_cost(frame: &serde_json::Value, changed: u64, retained: u64) {
    assert_eq!(frame["cost"]["delta_rows_carried"], changed * 4);
    assert_eq!(frame["cost"]["draw_list_mutations"], changed);
    assert_eq!(frame["cost"]["order_mutations"], changed);
    assert_eq!(frame["cost"]["logical_damage_regions"], 1);
    assert_eq!(
        frame["cost"]["damage_index_probes"],
        retained.saturating_mul(2).saturating_sub(1)
    );
    assert_eq!(frame["cost"]["damage_index_stored_records"], retained);
    assert_eq!(frame["cost"]["damage_index_high_water"], 2_048);
    assert_eq!(frame["cost"]["damage_region_command_checks"], retained);
    assert_eq!(frame["cost"]["intersecting_commands"], retained);
    assert_eq!(frame["cost"]["replayed_commands"], retained);
    assert_eq!(frame["cost"]["retained_command_scans"], 0);
    assert_eq!(frame["cost"]["queue_submissions"], 1);
    assert_eq!(frame["cost"]["presents"], 1);
}
