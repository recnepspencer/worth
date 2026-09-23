use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::native_platform::{CertifiedNativePlatform, NativePlatformContract};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

struct ControlManifest {
    logical_client_dimensions: [u32; 2],
    native_seed_control_point: Vec<ControlPoint>,
}

struct ControlPoint {
    name: String,
    logical: [u32; 2],
    snapshot_rgba: [u8; 4],
    external: ExternalControlExpectation,
}

enum ExternalControlExpectation {
    ExactRgb([u8; 3]),
    ObservedCompositorBackdrop,
}

#[derive(Default)]
struct PendingControlPoint {
    name: Option<String>,
    logical: Option<[u32; 2]>,
    snapshot_rgba: Option<[u8; 4]>,
    external_rgb: Option<[u8; 3]>,
    external_posture: Option<String>,
}

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn certified_native_boundary_world_correlates_presented_source_snapshot_and_client_pixels() {
    let controls = control_manifest();
    let argument = controls
        .native_seed_control_point
        .iter()
        .map(|control| format!("{},{}", control.logical[0], control.logical[1]))
        .collect::<Vec<_>>()
        .join(";");
    let platform = CertifiedNativePlatform::certified().expect("native observation is qualified");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(|product| product.launch_native_phase7(&argument))
        .expect("the native Phase 7 product process launches");
    let process_id = launch.process.id();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute(&platform, &mut launch, process_id, &controls)
    }));
    if let Err(failure) = outcome {
        finalize_failed_world(&platform, &mut launch.process, process_id);
        resume_unwind(failure);
    }
}

fn execute(
    platform: &CertifiedNativePlatform,
    launch: &mut crate::product_process::NativePhase2ProcessLaunch,
    process_id: u32,
    controls: &ControlManifest,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    let client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("one process-owned native client area appears");
    let bound = platform
        .observe_bound_client_area(&client)
        .expect("the Phase 7 client area remains process-bound");
    let external = await_control_pixels(platform, &client, controls, bound.dpi(), deadline);
    platform
        .request_normal_close(&client)
        .expect("the native window accepts normal close");
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("the Phase 7 process exits successfully");
    assert!(exit.status().success());
    platform
        .verify_process_window_released(process_id)
        .expect("the process leaves no native window");

    let mut stdout = String::new();
    launch
        .stdout
        .read_to_string(&mut stdout)
        .expect("Phase 7 evidence is readable");
    let evidence: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Phase 7 evidence is JSON");
    assert_evidence(&evidence, controls, bound, &external);
}

fn await_control_pixels(
    platform: &CertifiedNativePlatform,
    client: &<CertifiedNativePlatform as NativePlatformContract>::BoundClientArea,
    controls: &ControlManifest,
    dpi: u32,
    deadline: Instant,
) -> crate::external_observation::NativeClientPixelCapture {
    loop {
        if let Ok(capture) = platform.capture_client_area(client) {
            let current = controls.native_seed_control_point.iter().all(|control| {
                let point = control
                    .logical
                    .map(|logical| project_with_os_dpi(logical, dpi));
                match control.external {
                    ExternalControlExpectation::ExactRgb(expected) => {
                        pixel(&capture, point)[..3] == expected
                    }
                    ExternalControlExpectation::ObservedCompositorBackdrop => true,
                }
            });
            if current {
                return capture;
            }
        }
        assert!(
            Instant::now() < deadline,
            "Phase 7 product pixels did not become current before the deadline"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn assert_evidence(
    evidence: &serde_json::Value,
    controls: &ControlManifest,
    bound: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase7-evidence-v1");
    let snapshot = &evidence["snapshot"];
    assert_snapshot_geometry(snapshot, controls, bound, external);
    assert_snapshot_affinity(snapshot, &evidence["presentation"]);
    assert_capture_resources(evidence);
    assert_control_samples(snapshot, controls, bound.dpi(), external);
}

fn assert_snapshot_geometry(
    snapshot: &serde_json::Value,
    controls: &ControlManifest,
    bound: crate::external_observation::ProcessBoundNativeClientAreaObservation,
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    assert_eq!(snapshot["relation"], "current");
    assert_eq!(snapshot["orientation"], "top-left-origin");
    assert_eq!(snapshot["rounding"], "pixel-center-nearest");
    assert_eq!(snapshot["pixel_color_space"], "srgb");
    let bounds = bound.bounds();
    assert_eq!(
        snapshot["native_client_origin"],
        serde_json::json!([bounds.left(), bounds.top()])
    );
    assert_eq!(
        snapshot["client_physical_dimensions"],
        serde_json::json!([external.width(), external.height()])
    );
    assert_eq!(
        snapshot["pixel_dimensions"],
        snapshot["client_physical_dimensions"]
    );
    assert_eq!(snapshot["pixel_stride"], u64::from(external.width()) * 4);
    assert_eq!(snapshot["pixel_byte_count"], external.rgba().len());
    let expected_dimensions = controls
        .logical_client_dimensions
        .map(|logical| project_with_os_dpi(logical, bound.dpi()));
    assert_eq!(
        [external.width(), external.height()],
        expected_dimensions,
        "the independently observed DPI must explain the real client dimensions"
    );
    assert_eq!(
        snapshot["viewport_logical_dimension_bits"],
        serde_json::json!(controls
            .logical_client_dimensions
            .map(|value| (value as f32).to_bits()))
    );
    assert_eq!(
        snapshot["scale_bits"],
        serde_json::json!([bound.dpi() as f32 / 96.0; 2].map(f32::to_bits))
    );
    assert_eq!(
        snapshot["translation_bits"],
        serde_json::json!([0_u32, 0_u32])
    );
}

fn assert_snapshot_affinity(snapshot: &serde_json::Value, presentation: &serde_json::Value) {
    assert_eq!(snapshot["frame"], presentation["frame"]);
    assert_eq!(
        snapshot["presentation_attempt"],
        presentation["presentation_attempt"]
    );
    assert_eq!(
        snapshot["semantic_surface"],
        presentation["semantic_surface"]
    );
    assert_eq!(snapshot["host_surface"], presentation["host_surface"]);
    assert_eq!(snapshot["binding"], presentation["binding"]);
    assert!(snapshot["identity"].as_u64().is_some_and(|value| value > 0));
    assert!(snapshot["presentation_epoch"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert_eq!(snapshot["visible_region_count"], 1);
    assert_eq!(snapshot["hit_test_region_count"], 0);
}

fn assert_capture_resources(evidence: &serde_json::Value) {
    assert_eq!(evidence["capture_resources"]["peak_readback_buffers"], 1);
    assert_eq!(evidence["capture_resources"]["peak_pending_submissions"], 1);
    assert_eq!(
        evidence["capture_resources"]["terminal_readback_buffers"],
        0
    );
    assert_eq!(
        evidence["capture_resources"]["terminal_pending_submissions"],
        0
    );
    assert_eq!(evidence["terminal_zero"], true);
}

fn assert_control_samples(
    snapshot: &serde_json::Value,
    controls: &ControlManifest,
    dpi: u32,
    external: &crate::external_observation::NativeClientPixelCapture,
) {
    let samples = snapshot["samples"].as_array().expect("sample array");
    assert_eq!(samples.len(), controls.native_seed_control_point.len());
    for (sample, control) in samples.iter().zip(&controls.native_seed_control_point) {
        assert_eq!(
            sample["logical"],
            serde_json::json!(control.logical),
            "{}",
            control.name
        );
        assert_eq!(
            sample["rgba"],
            serde_json::json!(control.snapshot_rgba),
            "{}",
            control.name
        );
        let physical: [u32; 2] = serde_json::from_value(sample["physical"].clone()).unwrap();
        let expected_physical = control
            .logical
            .map(|logical| project_with_os_dpi(logical, dpi));
        assert_eq!(physical, expected_physical, "{}", control.name);
        let external_pixel = pixel(external, expected_physical);
        match control.external {
            ExternalControlExpectation::ExactRgb(expected) => {
                assert_eq!(
                    &external_pixel[..3],
                    expected.as_slice(),
                    "{}",
                    control.name
                );
            }
            ExternalControlExpectation::ObservedCompositorBackdrop => {
                assert_eq!(control.snapshot_rgba[3], 0, "{}", control.name);
            }
        }
    }
}

fn pixel(
    capture: &crate::external_observation::NativeClientPixelCapture,
    point: [u32; 2],
) -> [u8; 4] {
    assert!(point[0] < capture.width() && point[1] < capture.height());
    let offset = ((point[1] * capture.width() + point[0]) * 4) as usize;
    capture.rgba()[offset..offset + 4].try_into().unwrap()
}

fn control_manifest() -> ControlManifest {
    parse_control_manifest(include_str!(
        "../adjudication/native_profile_control_points.toml"
    ))
    .expect("the versioned native-seed control-point records are valid")
}

fn parse_control_manifest(source: &str) -> Option<ControlManifest> {
    if !source
        .lines()
        .any(|line| line.trim() == "world_version = 1")
    {
        return None;
    }
    let mut controls = Vec::new();
    let mut logical_client_dimensions = None;
    let mut pending = None;
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line.starts_with("[[") {
            finish_control(&mut controls, pending.take())?;
            if line == "[[native_seed_control_point]]" {
                pending = Some(PendingControlPoint::default());
            }
            continue;
        }
        if pending.is_none() && line.starts_with("logical_client_dimensions") {
            let (_, value) = line.split_once('=')?;
            logical_client_dimensions = Some(serde_json::from_str(value.trim()).ok()?);
            continue;
        }
        let Some(control) = pending.as_mut() else {
            continue;
        };
        let (field, value) = line.split_once('=')?;
        match field.trim() {
            "name" if control.name.is_none() => {
                control.name = Some(serde_json::from_str(value.trim()).ok()?);
            }
            "logical" if control.logical.is_none() => {
                control.logical = Some(serde_json::from_str(value.trim()).ok()?);
            }
            "snapshot_rgba" if control.snapshot_rgba.is_none() => {
                control.snapshot_rgba = Some(serde_json::from_str(value.trim()).ok()?);
            }
            "external_rgb" if control.external_rgb.is_none() => {
                control.external_rgb = Some(serde_json::from_str(value.trim()).ok()?);
            }
            "external_posture" if control.external_posture.is_none() => {
                control.external_posture = Some(serde_json::from_str(value.trim()).ok()?);
            }
            _ => return None,
        }
    }
    finish_control(&mut controls, pending)?;
    (!controls.is_empty()).then_some(ControlManifest {
        logical_client_dimensions: logical_client_dimensions?,
        native_seed_control_point: controls,
    })
}

fn finish_control(
    controls: &mut Vec<ControlPoint>,
    pending: Option<PendingControlPoint>,
) -> Option<()> {
    let Some(pending) = pending else {
        return Some(());
    };
    let external = match (pending.external_rgb, pending.external_posture.as_deref()) {
        (Some(rgb), None) => ExternalControlExpectation::ExactRgb(rgb),
        (None, Some("observed-compositor-backdrop")) => {
            ExternalControlExpectation::ObservedCompositorBackdrop
        }
        _ => return None,
    };
    controls.push(ControlPoint {
        name: pending.name?,
        logical: pending.logical?,
        snapshot_rgba: pending.snapshot_rgba?,
        external,
    });
    Some(())
}

fn project_with_os_dpi(logical: u32, dpi: u32) -> u32 {
    let numerator = u64::from(logical) * u64::from(dpi) + 48;
    u32::try_from(numerator / 96).expect("qualified logical extent projects into u32")
}

fn finalize_failed_world(
    platform: &CertifiedNativePlatform,
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
