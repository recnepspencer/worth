use std::process::ExitCode;

use worth_ui_native_platform::{
    UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT;

pub(crate) fn run() -> ExitCode {
    let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
        "WORTH UI Platform Pulse Phase 8",
        PLATFORM_PULSE_NATIVE_WINDOW_LOGICAL_EXTENT,
    ));
    let Ok(platform) = WorthUiNativePlatform::prepare(profile) else {
        return ExitCode::from(2);
    };
    let application = worth_ui_platform_pulse::PlatformPulseNativeSeedApplication::new()
        .with_surface_successor_capture();
    match platform.run(application) {
        UiNativePlatformOutcome::Closed(receipt) if receipt.terminal_census().is_zero() => {
            let Some(evidence) = crate::native_phase8_evidence::evidence(&receipt) else {
                return ExitCode::from(3);
            };
            println!("{evidence}");
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("worth-ui-native-phase8 stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}
