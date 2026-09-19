mod application;
mod launch_configuration;
mod lifecycle_observation_publication;
mod native_application;
#[cfg(feature = "executable-world")]
mod native_close_evidence;
#[cfg(feature = "executable-world")]
mod native_gate_d_application;
mod native_phase2_evidence;
#[cfg(feature = "executable-world")]
mod native_phase3_application;
mod native_phase6_evidence;
mod native_phase7_evidence;
#[cfg(feature = "executable-world")]
mod native_phase8_evidence;
#[cfg(feature = "executable-world")]
mod native_phase8_world;
#[cfg(feature = "executable-world")]
mod native_phase_f_application;
#[cfg(feature = "executable-world")]
mod native_phase_f_cancellation_world;
#[cfg(feature = "executable-world")]
mod native_phase_f_deferred_completion_world;
#[cfg(feature = "executable-world")]
mod native_phase_f_evidence;
#[cfg(feature = "executable-world")]
mod native_phase_f_reconstruction_world;
#[cfg(feature = "executable-world")]
mod native_phase_f_world;
#[cfg(feature = "executable-world")]
mod native_phase_f_world_evidence;
mod product_process;
mod query_source;
mod source_watch;
mod theme_preference;
mod visual_identity_adjudication;
mod visual_identity_execution;
mod visual_observation_publication;

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Some(points) = std::env::args().find_map(|argument| {
        argument
            .strip_prefix("--worth-ui-native-phase7-world=")
            .and_then(native_phase7_evidence::parse_control_points)
    }) {
        return run_native_phase7_world(&points);
    }
    #[cfg(feature = "executable-world")]
    if let Some(class) = std::env::args().find_map(|argument| {
        argument
            .strip_prefix("--worth-ui-native-phase-f-reconstruction-world=")
            .map(str::to_owned)
    }) {
        return native_phase_f_reconstruction_world::run(&class);
    }
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-phase2-world") {
        return run_native_phase2_world();
    }
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-phase6-world") {
        return run_native_phase6_world();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-phase8-world") {
        return native_phase8_world::run();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-phase3-world") {
        return run_native_phase3_world();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-gate-d-pin-world") {
        return run_native_gate_d_pin_world();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os()
        .any(|argument| argument == "--worth-ui-native-phase-f-partial-cancellation-world")
    {
        return native_phase_f_cancellation_world::run();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os()
        .any(|argument| argument == "--worth-ui-native-phase-f-deferred-completion-world")
    {
        return native_phase_f_deferred_completion_world::run();
    }
    #[cfg(feature = "executable-world")]
    if std::env::args_os().any(|argument| argument == "--worth-ui-native-phase-f-world") {
        return native_phase_f_world::run();
    }
    product_process::run()
}

#[cfg(feature = "executable-world")]
fn run_native_gate_d_pin_world() -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Gate D Pin Courtroom",
        [160, 96],
    ));
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let Some(presentation_async) = query_source::install_native_presentation_async() else {
        return ExitCode::from(2);
    };
    match platform.run(
        native_gate_d_application::PlatformPulseNativeGateDApplication::new(presentation_async),
    ) {
        UiNativePlatformOutcome::Closed(receipt) => {
            let peak = receipt.peak_census();
            let terminal = receipt.terminal_census();
            let Some(client_shutdown) = receipt.client_shutdown() else {
                return ExitCode::from(3);
            };
            let pin_frames = receipt
                .text_pin_frame_observations()
                .iter()
                .map(|frame| {
                    frame
                        .iter()
                        .map(|pin| {
                            serde_json::json!({
                                "layout": hex_digest(pin.layout_digest()),
                                "raster_key": hex_digest(pin.raster_key_digest()),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let evidence = serde_json::json!({
                "schema": "worth-ui-native-gate-d-pin-world-v3",
                "mounted_bindings": usize::from(receipt.presentation().is_some_and(|presentation| presentation.binding_generation() != 0)),
                "pinned_layouts": receipt.peak_text_layout_count(),
                "presentations": receipt.retained_frames().len(),
                "atlas_transactions": receipt.text_atlas_transactions(),
                "native_peak_pin_count": peak.text_atlas_pins,
                "native_frame_pin_counts": receipt.text_pin_frame_counts(),
                "native_frame_pins": pin_frames,
                "observation_history_complete": receipt.observation_history_complete(),
                "physical_signal_runtimes": peak.physical_signal_runtimes,
                "physical_signal_workers": peak.physical_signal_workers,
                "alpha_entries": peak.text_atlas_alpha_entries,
                "color_entries": peak.text_atlas_color_entries,
                "closed_query_resources": client_shutdown.managed_semantic_resources_closed(),
                "query_close_complete": client_shutdown.managed_semantic_resources_complete(),
                "terminal_zero": terminal.is_zero(),
            });
            println!("{evidence}");
            if terminal.is_zero()
                && peak.text_atlas_pins > 0
                && receipt.observation_history_complete()
                && client_shutdown.managed_semantic_resources_complete()
                && client_shutdown.managed_semantic_resources_closed() > 0
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(3)
            }
        }
        outcome => {
            eprintln!("worth-ui-native-gate-d stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}

#[cfg(feature = "executable-world")]
fn hex_digest(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(feature = "executable-world")]
fn run_native_phase3_world() -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Platform Pulse Phase 3",
        [160, 96],
    ));
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let outcome = platform.run(native_phase3_application::PlatformPulseNativePhase3Application);
    match outcome {
        UiNativePlatformOutcome::Closed(receipt) if receipt.terminal_census().is_zero() => {
            println!("{}", native_phase3_evidence(&receipt));
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("worth-ui-native-phase3 stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}

#[cfg(feature = "executable-world")]
fn native_phase3_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    let frames = receipt
        .retained_frames()
        .iter()
        .map(|frame| {
            serde_json::json!({
                "frame": frame.frame(),
                "kind": format!("{:?}", frame.kind()),
                "baseline": frame.retained_baseline_rgba8(),
                "center": frame.retained_center_rgba8(),
                "cost": phase3_frame_cost(frame.cost()),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema": "worth-ui-native-phase3-evidence-v1",
        "frames": frames,
        "terminal_zero": receipt.terminal_census().is_zero(),
    })
}

#[cfg(feature = "executable-world")]
fn phase3_frame_cost(
    cost: worth_ui::facade::app::UiHostPresentationCostReport,
) -> serde_json::Value {
    serde_json::json!({
        "delta_rows_carried": cost.delta_rows_carried(),
        "draw_list_mutations": cost.draw_list_mutations(),
        "order_mutations": cost.order_mutations(),
        "logical_damage_regions": cost.logical_damage_regions(),
        "retained_command_scans": cost.retained_command_scans(),
        "intersecting_commands": cost.intersecting_commands(),
        "replayed_commands": cost.replayed_commands(),
        "damage_region_command_checks": cost.damage_region_command_checks(),
        "damage_index_probes": cost.damage_index_probes(),
        "damage_index_stored_records": cost.damage_index_stored_records(),
        "damage_index_high_water": cost.damage_index_high_water(),
        "cleared_pixels": cost.cleared_pixels(),
        "rendered_pixels": cost.rendered_pixels(),
        "presented_pixels": cost.presented_pixels(),
        "queue_submissions": cost.queue_submissions(),
        "surface_acquisitions": cost.surface_acquisitions(),
        "surface_copies": cost.surface_copies(),
        "gpu_writes": cost.gpu_writes(),
        "render_passes": cost.render_passes(),
        "presents": cost.presents(),
    })
}

fn run_native_phase2_world() -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    let window = UiNativeWindowSpec::new("WORTH UI Platform Pulse", [160, 96]);
    let profile = UiNativePlatformProfile::single_window(window);
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    match platform.run(worth_ui_platform_pulse::PlatformPulseNativeSeedApplication::new()) {
        UiNativePlatformOutcome::Closed(receipt)
            if receipt.terminal_census().is_zero()
                && receipt.final_frame().retained_center_rgba8() == [47, 129, 247, 255]
                && receipt.final_frame().retained_baseline_rgba8() == [0, 0, 0, 0] =>
        {
            println!(
                "{}",
                native_phase2_evidence::native_phase2_evidence(&receipt)
            );
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("worth-ui-native-phase2 stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}

fn run_native_phase6_world() -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Platform Pulse Phase 6",
        [160, 96],
    ));
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    match platform.run(worth_ui_platform_pulse::PlatformPulseNativeSeedApplication::new()) {
        UiNativePlatformOutcome::Closed(receipt)
            if receipt.terminal_census().is_zero()
                && receipt.input_observations().retained_event_count() > 0
                && receipt.input_observations().last_pointer_button().is_some()
                && receipt.client_shutdown().is_some_and(|shutdown| {
                    let counts = shutdown.observation_ingress().counts();
                    counts[0] > 0
                        && counts[4] == 0
                        && counts[..4].iter().sum::<u64>()
                            >= receipt.input_observations().retained_batch_count()
                }) =>
        {
            println!(
                "{}",
                native_phase6_evidence::native_phase6_evidence(&receipt)
            );
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("worth-ui-native-phase6 stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}

fn run_native_phase7_world(points: &[[u32; 2]]) -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Platform Pulse Phase 7",
        [160, 96],
    ));
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let application = worth_ui_platform_pulse::PlatformPulseNativeSeedApplication::new()
        .with_presented_source_capture();
    match platform.run(application) {
        UiNativePlatformOutcome::Closed(receipt) if receipt.terminal_census().is_zero() => {
            let Some(evidence) = native_phase7_evidence::evidence(&receipt, points) else {
                return ExitCode::from(3);
            };
            println!("{evidence}");
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("worth-ui-native-phase7 stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}
