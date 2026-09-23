use std::time::{Duration, Instant};

use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

const CLASSES: [&str; 7] = [
    "layout",
    "raster",
    "atlas",
    "pins",
    "draw-list",
    "target",
    "affinity",
];

#[test]
#[ignore = "requires the serialized certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn every_derived_state_reconstructs_in_a_fresh_product_world() {
    let portfolio_deadline = Instant::now() + Duration::from_secs(510);
    for class in CLASSES {
        let started = Instant::now();
        let mut launch = CargoBuiltPlatformPulse::exact()
            .and_then(|binary| binary.launch_native_phase_f_reconstruction(class))
            .unwrap_or_else(|denial| panic!("{class} reconstruction world launch: {denial}"));
        let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, portfolio_deadline)
            .unwrap_or_else(|denial| panic!("{class} reconstruction world exit: {denial}"));
        assert!(
            exit.status().success(),
            "{class} reconstruction world failed"
        );
        let mut stdout = String::new();
        launch.stdout.read_to_string(&mut stdout).unwrap();
        let evidence: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
        assert_eq!(
            evidence["schema"],
            "worth-ui-native-phase-f-reconstruction-world-v1"
        );
        assert_eq!(evidence["loss_count"], 1);
        assert_eq!(evidence["reconstruction_count"], 1);
        assert_eq!(evidence["reconstructed_frames"], 1);
        assert_eq!(evidence["reconstruction_pixels_exact"], true);
        assert_eq!(evidence["reconstruction_native_transcript_exact"], true);
        assert_eq!(evidence["reconstruction_headless_transcript_exact"], true);
        assert_eq!(evidence["reconstruction_atlas_model_exact"], true);
        assert_eq!(evidence["next_delta_local"], true);
        assert_eq!(evidence["terminal_zero"], true);
        assert_eq!(evidence["query_close_complete"], true);
        eprintln!(
            "phase5-reconstruction class={class} elapsed_ms={}",
            started.elapsed().as_millis()
        );
    }
}
