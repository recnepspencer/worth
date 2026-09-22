use std::process::ExitCode;

use worth_ui_native_platform::{
    UiNativeDerivedStateLossClass as LossClass, UiNativePlatformCloseReceipt,
    UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeQualificationPlan,
    UiNativeRuntimeDerivedStateLossClass, UiNativeRuntimeQualificationPlan, UiNativeWindowSpec,
    WorthUiNativePlatform,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT;

mod exact_reconstruction;
use exact_reconstruction::{
    atlas_reconstruction_exact, headless_reconstruction_exact, next_delta_is_local,
    reconstruction_frames,
};

#[derive(Clone, Copy, Debug)]
enum ReconstructionScenario {
    Host(LossClass),
    Runtime(UiNativeRuntimeDerivedStateLossClass),
}

struct ReconstructionWorldJudgement {
    evidence: serde_json::Value,
    accepted: bool,
}

#[derive(Clone, Copy)]
struct ReconstructionCounts {
    loss: u64,
    reconstruction: u64,
    derived_items_lost: u64,
    derived_items_reconstructed: u64,
}

struct ReconstructionFrameEvidence {
    reconstructed_frames: usize,
    pixels_exact: bool,
    native_transcript_exact: bool,
    predecessor_request: Option<[u64; 3]>,
    reconstructed_request: Option<[u64; 3]>,
    next_delta_request: Option<[u64; 3]>,
    predecessor_pixels: Option<[[u8; 4]; 2]>,
    reconstructed_pixels: Option<[[u8; 4]; 2]>,
    retained_frames: Vec<serde_json::Value>,
}

struct ReconstructionEvidenceDetails {
    counts: ReconstructionCounts,
    frames: ReconstructionFrameEvidence,
    next_delta_local: bool,
    headless_transcript_exact: bool,
    atlas_model_exact: bool,
}

pub(crate) fn run(class_name: &str) -> ExitCode {
    let Some(scenario) = parse_class(class_name) else {
        eprintln!("unknown Phase F reconstruction class: {class_name}");
        return ExitCode::from(2);
    };
    let receipt = match execute_world(scenario) {
        Ok(receipt) => receipt,
        Err(denial) => {
            eprintln!("Phase F reconstruction world stopped: {denial}");
            return ExitCode::from(3);
        }
    };
    let judgement = match adjudicate(scenario, &receipt) {
        Ok(judgement) => judgement,
        Err(denial) => {
            eprintln!("Phase F reconstruction evidence denied: {denial}");
            return ExitCode::from(3);
        }
    };
    println!("{}", judgement.evidence);
    if judgement.accepted {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

fn execute_world(scenario: ReconstructionScenario) -> Result<UiNativePlatformCloseReceipt, String> {
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Phase F Reconstruction Courtroom",
        PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT,
    ));
    let profile = match scenario {
        ReconstructionScenario::Host(class) => profile.with_native_qualification_plan(
            UiNativeQualificationPlan::derived_state_loss_after_completed_presentation(1, class)
                .expect("the first completed presentation is a bounded qualification point"),
        ),
        ReconstructionScenario::Runtime(class) => profile.with_runtime_qualification_plan(
            UiNativeRuntimeQualificationPlan::derived_state_loss_after_completed_presentation(
                1, class,
            )
            .expect("the first completed presentation is a bounded qualification point"),
        ),
    };
    let platform = WorthUiNativePlatform::prepare(profile)
        .map_err(|denial| format!("platform preparation: {denial:?}"))?;
    let presentation_async = crate::query_source::install_native_presentation_async()
        .ok_or_else(|| "presentation async installation".to_owned())?;
    let application = crate::native_phase_f_application::PlatformPulseNativePhaseFApplication::new(
        presentation_async,
    )
    .reconstruct_current_frame();
    match platform.run(application) {
        UiNativePlatformOutcome::Closed(receipt) => Ok(receipt),
        outcome => Err(format!("platform outcome {outcome:?}")),
    }
}

fn adjudicate(
    scenario: ReconstructionScenario,
    receipt: &UiNativePlatformCloseReceipt,
) -> Result<ReconstructionWorldJudgement, String> {
    let shutdown = receipt
        .client_shutdown()
        .ok_or_else(|| "world closed without client shutdown evidence".to_owned())?;
    let host_observation = receipt.derived_state_reconstruction();
    let runtime_observation = shutdown.derived_state_reconstruction();
    let counts = reconstruction_counts(scenario, host_observation, runtime_observation);
    let frames = reconstruction_frames(receipt);
    let headless_transcript_exact = headless_reconstruction_exact(shutdown, &frames);
    let atlas_model_exact = atlas_reconstruction_exact(receipt, shutdown, &frames);
    let next_delta_local = next_delta_is_local(shutdown, &frames);
    let peak_census = receipt.peak_census();
    let resources_observed =
        peak_census.client_mounted_layouts > 0 && peak_census.client_raster_cache_entries > 0;
    let exact_observation = observation_matches(scenario, host_observation, runtime_observation);
    let accepted = exact_observation
        && frames.reconstructed_frames == 1
        && frames.pixels_exact
        && frames.native_transcript_exact
        && headless_transcript_exact
        && atlas_model_exact
        && next_delta_local
        && resources_observed
        && shutdown.managed_semantic_resources_complete()
        && shutdown.managed_semantic_resources_closed() > 0
        && receipt.terminal_census().is_zero()
        && receipt.observation_history_complete();
    let details = ReconstructionEvidenceDetails {
        counts,
        frames,
        next_delta_local,
        headless_transcript_exact,
        atlas_model_exact,
    };
    Ok(ReconstructionWorldJudgement {
        evidence: evidence_row(scenario, receipt, shutdown, details),
        accepted,
    })
}

fn reconstruction_counts(
    scenario: ReconstructionScenario,
    host: Option<worth_ui_native_platform::UiNativeDerivedStateReconstructionObservation>,
    runtime: Option<worth_ui_native_platform::UiNativeClientDerivedStateReconstructionObservation>,
) -> ReconstructionCounts {
    match scenario {
        ReconstructionScenario::Host(_) => ReconstructionCounts {
            loss: host.map_or(0, |value| value.loss_count()),
            reconstruction: host.map_or(0, |value| value.reconstruction_count()),
            derived_items_lost: 0,
            derived_items_reconstructed: 0,
        },
        ReconstructionScenario::Runtime(_) => ReconstructionCounts {
            loss: runtime.map_or(0, |value| value.loss_count()),
            reconstruction: runtime.map_or(0, |value| value.reconstruction_count()),
            derived_items_lost: runtime.map_or(0, |value| value.derived_items_lost()),
            derived_items_reconstructed: runtime
                .map_or(0, |value| value.derived_items_reconstructed()),
        },
    }
}

fn evidence_row(
    scenario: ReconstructionScenario,
    receipt: &UiNativePlatformCloseReceipt,
    shutdown: &worth_ui_native_platform::UiNativeClientShutdownObservation,
    details: ReconstructionEvidenceDetails,
) -> serde_json::Value {
    let peak_census = receipt.peak_census();
    serde_json::json!({
        "schema": "worth-ui-native-phase-f-reconstruction-world-v1",
        "class": format!("{scenario:?}"),
        "loss_count": details.counts.loss,
        "reconstruction_count": details.counts.reconstruction,
        "derived_items_lost": details.counts.derived_items_lost,
        "derived_items_reconstructed": details.counts.derived_items_reconstructed,
        "reconstructed_frames": details.frames.reconstructed_frames,
        "reconstruction_pixels_exact": details.frames.pixels_exact,
        "reconstruction_native_transcript_exact": details.frames.native_transcript_exact,
        "reconstruction_headless_transcript_exact": details.headless_transcript_exact,
        "reconstruction_atlas_model_exact": details.atlas_model_exact,
        "predecessor_pixels": details.frames.predecessor_pixels,
        "reconstructed_pixels": details.frames.reconstructed_pixels,
        "predecessor_request": details.frames.predecessor_request,
        "reconstructed_request": details.frames.reconstructed_request,
        "next_delta_request": details.frames.next_delta_request,
        "retained_frames": details.frames.retained_frames,
        "next_delta_local": details.next_delta_local,
        "peak_mounted_layouts": peak_census.client_mounted_layouts,
        "peak_raster_cache_entries": peak_census.client_raster_cache_entries,
        "atlas_plans": receipt.text_atlas_plan_observations().len(),
        "text_pin_frame_counts": receipt.text_pin_frame_counts(),
        "text_atlas_model_frame_digests": receipt.text_atlas_model_frame_digests().iter().copied().map(crate::native_phase_f_evidence::hex_digest).collect::<Vec<_>>(),
        "text_work": shutdown.text_presentation_work().iter().map(|work| serde_json::json!({
            "attempt": work.attempt(),
            "binding": work.binding(),
            "mounted_frame": work.mounted_frame(),
            "host_lineage": work.host_lineage(),
            "demand_records": work.demand_records(),
            "rasterized_glyphs": work.rasterized_glyphs(),
            "pin_additions": work.pin_additions(),
            "binding_pins": work.binding_pins(),
            "layout_set_digest": crate::native_phase_f_evidence::hex_digest(work.layout_set_digest()),
            "raster_key_set_digest": crate::native_phase_f_evidence::hex_digest(work.raster_key_set_digest()),
            "glyph_run_transcript_digest": crate::native_phase_f_evidence::hex_digest(work.glyph_run_transcript_digest()),
            "intrinsic_glyph_transcript_digest": crate::native_phase_f_evidence::hex_digest(work.intrinsic_glyph_transcript_digest()),
        })).collect::<Vec<_>>(),
        "query_close_complete": shutdown.managed_semantic_resources_complete(),
        "closed_query_resources": shutdown.managed_semantic_resources_closed(),
        "terminal_zero": receipt.terminal_census().is_zero(),
        "observation_history_complete": receipt.observation_history_complete(),
    })
}

fn observation_matches(
    scenario: ReconstructionScenario,
    host: Option<worth_ui_native_platform::UiNativeDerivedStateReconstructionObservation>,
    runtime: Option<worth_ui_native_platform::UiNativeClientDerivedStateReconstructionObservation>,
) -> bool {
    match scenario {
        ReconstructionScenario::Host(class) => host.is_some_and(|value| {
            value.class() == class && value.loss_count() == 1 && value.reconstruction_count() == 1
        }),
        ReconstructionScenario::Runtime(class) => runtime.is_some_and(|value| {
            let expected = match class {
                UiNativeRuntimeDerivedStateLossClass::MountedLayouts => {
                    worth_ui_native_platform::UiNativeClientDerivedStateLossClass::MountedLayouts
                }
                UiNativeRuntimeDerivedStateLossClass::RasterCache => {
                    worth_ui_native_platform::UiNativeClientDerivedStateLossClass::RasterCache
                }
            };
            value.class() == expected
                && value.loss_count() == 1
                && value.reconstruction_count() == 1
                && value.derived_items_lost() > 0
                && value.derived_items_lost() == value.derived_items_reconstructed()
        }),
    }
}

fn parse_class(name: &str) -> Option<ReconstructionScenario> {
    match name {
        "layout" => Some(ReconstructionScenario::Runtime(
            UiNativeRuntimeDerivedStateLossClass::MountedLayouts,
        )),
        "raster" => Some(ReconstructionScenario::Runtime(
            UiNativeRuntimeDerivedStateLossClass::RasterCache,
        )),
        "atlas" => Some(ReconstructionScenario::Host(
            LossClass::TextAtlasPagesAndIndex,
        )),
        "pins" => Some(ReconstructionScenario::Host(LossClass::TextAtlasPins)),
        "draw-list" => Some(ReconstructionScenario::Host(LossClass::RetainedDrawList)),
        "target" => Some(ReconstructionScenario::Host(LossClass::RetainedTarget)),
        "affinity" => Some(ReconstructionScenario::Host(
            LossClass::PresentationAffinity,
        )),
        _ => None,
    }
}
