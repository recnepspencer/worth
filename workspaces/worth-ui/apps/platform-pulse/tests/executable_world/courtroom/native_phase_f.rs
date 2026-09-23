use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::native_platform::{CertifiedNativePlatform, NativePlatformContract};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

#[path = "native_phase_f/authored_pixel_contract.rs"]
mod authored_pixel_contract;
#[path = "native_phase_f/lineage.rs"]
mod lineage;
#[path = "native_phase_f/physical_trace.rs"]
mod physical_trace;
#[path = "native_phase_f/pixels.rs"]
mod pixels;
use pixels::{attributed_pixel_classes, pixel_classes};

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn query_async_reconstruction_joins_exact_transitions_to_external_pixels_and_cleanup() {
    let platform = CertifiedNativePlatform::certified().expect("native observation is qualified");
    let binary =
        CargoBuiltPlatformPulse::exact().expect("the prebuilt Phase F product binary is available");
    let mut launch = binary
        .clone()
        .launch_native_phase_f_world()
        .expect("the Phase F product world launches under the desktop lease");
    let process_id = launch.process.id();
    let deadline = Instant::now() + Duration::from_secs(120);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute_world(&platform, &mut launch, process_id, deadline)
    }));
    outcome.unwrap_or_else(|failure| {
        let teardown = launch
            .process
            .terminate_after_failure(Instant::now() + Duration::from_secs(5));
        let release = platform.verify_process_window_released(process_id);
        assert!(teardown.is_ok() && release.is_ok());
        resume_unwind(failure)
    });
    drop(launch);
    assert_partial_effects_cancellation_world(&binary);
}

fn execute_world(
    platform: &CertifiedNativePlatform,
    launch: &mut crate::product_process::NativePhase2ProcessLaunch,
    process_id: u32,
    deadline: Instant,
) {
    let mut client = platform
        .bind_process_client_area(process_id, deadline)
        .expect("the Phase F client area appears");
    let bound = platform
        .observe_bound_client_area(&client)
        .expect("the Phase F client area remains process-bound");
    assert_eq!(bound.process_id(), process_id);
    let pixel_deadline = Instant::now() + Duration::from_secs(10);
    let (capture, authored_pixels_proved) =
        await_quiescent_text_pixels(platform, &mut client, pixel_deadline);
    platform
        .request_normal_close(&client)
        .expect("the external courtroom closes the product window");
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline);
    platform
        .verify_process_window_released(process_id)
        .expect("the Phase F process leaves no native window");
    let mut stdout = String::new();
    launch.stdout.read_to_string(&mut stdout).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let exit = exit.unwrap_or_else(|failure| {
        panic!("the Phase F product exits without residue: {failure}; evidence={evidence}")
    });
    assert!(exit.status().success());
    let capture = capture.unwrap_or_else(|| {
        panic!("the Phase F client area produced no capture; evidence={evidence}")
    });
    assert!(
        authored_pixels_proved,
        "the Phase F product did not expose alpha and intrinsic-color text pixels; pixels={:?}; evidence={evidence}",
        pixel_classes(&capture)
    );
    assert_alpha_attribution(&evidence);
    assert_intrinsic_attribution(&evidence);
    assert_headless_glyph_agreement(&evidence);
    let authored = authored_pixel_contract::assert_owner_projection(&evidence);
    let pixels =
        attributed_pixel_classes(&capture, &authored.alpha_bounds, &authored.intrinsic_bounds);
    assert_phase_f_evidence(&evidence, &capture, &pixels);
}

fn assert_partial_effects_cancellation_world(binary: &CargoBuiltPlatformPulse) {
    let mut launch = binary
        .clone()
        .launch_native_phase_f_partial_cancellation()
        .expect("the real native partial-effects cancellation world launches");
    let deadline = Instant::now() + Duration::from_secs(120);
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("the partial-effects cancellation world exits without residue");
    let mut stdout = String::new();
    launch.stdout.read_to_string(&mut stdout).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(exit.status().success(), "cancellation evidence={evidence}");
    assert_eq!(
        evidence["schema"],
        "worth-ui-native-phase-f-partial-cancellation-world-v1"
    );
    assert_eq!(evidence["exact_query_recovery"], true);
    assert_eq!(evidence["exact_cancelled_physical_request"], true);
    assert_eq!(evidence["physical_signal"]["cancellations"], 1);
    assert_eq!(evidence["physical_signal"]["recovery_schedules"], 1);
    assert_eq!(evidence["physical_signal"]["recovery_resolutions"], 1);
    assert_eq!(
        evidence["physical_signal"]["transition_trace_complete"],
        true
    );
    assert_eq!(evidence["query_close_complete"], true);
    assert_eq!(evidence["terminal_zero"], true);
}

fn assert_alpha_attribution(evidence: &serde_json::Value) {
    let glyphs = evidence["presentation"]["alpha_glyphs"]
        .as_array()
        .expect("the retained native presentation carries qualified alpha glyphs");
    assert!(!glyphs.is_empty());
    for glyph in glyphs {
        assert!(matches!(
            glyph["source"].as_str(),
            Some("AlphaOutline" | "LastResort")
        ));
        assert_eq!(glyph["foreground"], serde_json::json!([255, 255, 255, 255]));
        assert_eq!(glyph["raster_key"].as_str().unwrap().len(), 64);
        assert_eq!(glyph["transcript_digest"].as_str().unwrap().len(), 64);
        assert_valid_bounds(&glyph["target_bounds"]);
    }
}

fn await_quiescent_text_pixels(
    platform: &CertifiedNativePlatform,
    client: &mut <CertifiedNativePlatform as NativePlatformContract>::BoundClientArea,
    deadline: Instant,
) -> (
    Option<crate::external_observation::NativeClientPixelCapture>,
    bool,
) {
    let mut previous: Option<Vec<u8>> = None;
    let mut last_capture = None;
    let mut capture_count = 0_u32;
    loop {
        if let Ok(capture) = platform.capture_client_area(client) {
            let colors = pixel_classes(&capture);
            capture_count = capture_count.saturating_add(1);
            if capture_count.is_multiple_of(10) {
                eprintln!("Phase F authored-pixel progress: {colors:?}");
            }
            if colors.proves_authored_text(capture.width(), capture.height()) {
                if previous
                    .as_ref()
                    .is_some_and(|prior| prior.as_slice() == capture.rgba())
                {
                    return (Some(capture), true);
                }
                previous = Some(capture.rgba().to_vec());
            }
            last_capture = Some(capture);
        }
        if Instant::now() >= deadline {
            return (last_capture, false);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn assert_phase_f_evidence(
    evidence: &serde_json::Value,
    capture: &crate::external_observation::NativeClientPixelCapture,
    pixels: &pixels::PixelClasses,
) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase-f-async-world-v1");
    assert_eq!(evidence["presentation_transition_count"], 11);
    let transitions = evidence["presentation_transitions"].as_array().unwrap();
    let kinds = transitions
        .iter()
        .map(|transition| transition["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            "Pending",
            "Superseded",
            "StaleCompletionRejected",
            "Pending",
            "Completed",
            "DuplicateCompletionRejected",
            "Pending",
            "Unresolved",
            "RecoveryRequired",
            "ReconstructionCurrent",
            "TerminalClosed",
        ]
    );
    assert_request_equal(&transitions[0], &transitions[2]);
    assert_ne!(transitions[1]["attempt"], transitions[3]["attempt"]);
    assert_eq!(transitions[1]["binding"], transitions[3]["binding"]);
    assert_request_equal(&transitions[3], &transitions[4]);
    assert_request_equal(&transitions[4], &transitions[5]);
    assert_request_equal(&transitions[6], &transitions[7]);
    assert_request_equal(&transitions[7], &transitions[8]);
    assert_request_equal(&transitions[9], &transitions[10]);
    assert_ne!(transitions[8]["binding"], transitions[9]["binding"]);
    physical_trace::assert_supersession(
        evidence,
        &transitions[0],
        &transitions[1],
        &transitions[3],
    );
    physical_trace::assert_duplicate_rejection(evidence, &transitions[5]);
    physical_trace::assert_indeterminate(evidence, &transitions[6]);
    lineage::assert_exact_request_lineage(evidence, &transitions[9]);
    assert_eq!(
        evidence["presentation"]["frame"],
        evidence["runtime_attribution"]["frame"]
    );
    assert_eq!(
        evidence["presentation"]["binding"],
        transitions[9]["binding"]
    );
    assert_eq!(
        evidence["presentation"]["attempt"],
        transitions[9]["attempt"]
    );
    assert!(evidence["retained_frames"]
        .as_array()
        .unwrap()
        .iter()
        .any(|frame| frame["kind"] == "Reconstruction"));
    assert!(pixels.proves_authored_text(capture.width(), capture.height()));
    assert_eq!(evidence["observation_history_complete"], true);
    assert_eq!(evidence["terminal_zero"], true);
    assert_eq!(evidence["query_close_complete"], true);
    assert!(evidence["closed_query_resources"].as_u64().unwrap() > 0);
}

fn assert_intrinsic_attribution(evidence: &serde_json::Value) {
    let glyphs = evidence["presentation"]["intrinsic_glyphs"]
        .as_array()
        .expect("the retained native presentation carries qualified intrinsic glyphs");
    assert!(!glyphs.is_empty());
    for glyph in glyphs {
        assert!(matches!(
            glyph["source"].as_str(),
            Some("ColorOutline" | "ColorBitmap")
        ));
        assert_eq!(glyph["original_range"], serde_json::json!([8, 19]));
        assert_eq!(glyph["palette"].as_u64(), Some(0));
        let key = glyph["raster_key"].as_str().unwrap();
        assert_eq!(key.len(), 64);
        assert_valid_bounds(&glyph["target_bounds"]);
    }
}

fn assert_valid_bounds(value: &serde_json::Value) {
    let coordinates = value.as_array().expect("glyph target bounds are present");
    let [left, top, right, bottom] = coordinates.as_slice() else {
        panic!("glyph target bounds must have four coordinates");
    };
    let left = left.as_u64().unwrap();
    let top = top.as_u64().unwrap();
    let right = right.as_u64().unwrap();
    let bottom = bottom.as_u64().unwrap();
    assert!(left < right && top < bottom);
}

fn assert_headless_glyph_agreement(evidence: &serde_json::Value) {
    let presentation = &evidence["presentation"];
    let native_digest = presentation["intrinsic_glyph_transcript_digest"]
        .as_str()
        .expect("native presentation retains its intrinsic transcript digest");
    let attempt = presentation["attempt"].as_u64().unwrap();
    let binding = presentation["binding"].as_u64().unwrap();
    let mounted_frame = presentation["frame"].as_u64().unwrap();
    let matches = evidence["text_presentation_work"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|work| {
            work["attempt"] == attempt
                && work["binding"] == binding
                && work["mounted_frame"] == mounted_frame
        })
        .collect::<Vec<_>>();
    let [headless] = matches.as_slice() else {
        panic!("the exact presented request must retain one headless transcript");
    };
    let headless_digest = headless["intrinsic_glyph_transcript_digest"]
        .as_str()
        .unwrap();
    let intrinsic_count = headless["intrinsic_glyph_runs"].as_u64().unwrap();
    let native_count = presentation["intrinsic_glyphs"].as_array().unwrap().len() as u64;
    assert!(pixels::headless_intrinsic_agrees(
        headless_digest,
        native_digest,
        intrinsic_count,
        native_count,
    ));
    assert_eq!(
        headless["glyph_run_transcript_digest"], presentation["glyph_transcript_digest"],
        "headless and native alpha+intrinsic glyph transcripts diverged",
    );
    for field in [
        "layout_set_digest",
        "raster_key_set_digest",
        "glyph_run_transcript_digest",
    ] {
        assert_eq!(headless[field].as_str().unwrap().len(), 64);
        assert_ne!(headless[field].as_str().unwrap(), "0".repeat(64));
    }
}

fn assert_request_equal(left: &serde_json::Value, right: &serde_json::Value) {
    assert_eq!(left["attempt"], right["attempt"]);
    assert_eq!(left["binding"], right["binding"]);
}
