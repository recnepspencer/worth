use std::process::ExitCode;

use crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration;
use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

pub(crate) fn run() -> ExitCode {
    let publisher = match PlatformPulseObservationPublisher::start() {
        Ok(publisher) => publisher,
        Err(denial) => {
            eprintln!("WORTH UI platform pulse observation stream could not start: {denial:?}");
            return ExitCode::FAILURE;
        }
    };
    #[cfg(feature = "executable-world")]
    let admitted_launch =
        AdmittedPlatformPulseLaunchConfiguration::from_arguments(std::env::args_os().skip(1));
    #[cfg(not(feature = "executable-world"))]
    let admitted_launch = AdmittedPlatformPulseLaunchConfiguration::from_process();
    let launch = match admitted_launch {
        Ok(launch) => launch,
        Err(denial) => {
            if let Err(publication) = publisher.launch_configuration_failure(&denial) {
                eprintln!(
                    "WORTH UI platform pulse launch denial could not be observed: {publication:?}"
                );
            }
            eprintln!("WORTH UI platform pulse launch was denied: {denial:?}");
            return ExitCode::from(2);
        }
    };
    run_worth_native(launch, publisher)
}

fn run_worth_native(
    launch: AdmittedPlatformPulseLaunchConfiguration,
    publisher: PlatformPulseObservationPublisher,
) -> ExitCode {
    use worth_ui_native_platform::{
        UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
    };
    use worth_ui_platform_pulse::visual_identity_pulse::{
        PLATFORM_PULSE_MINIMUM_LOGICAL_EXTENT, PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT,
    };
    let profile = UiNativePlatformProfile::single_window(
        UiNativeWindowSpec::new(
            "WORTH UI Platform Pulse",
            PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT,
        )
        .with_minimum_logical_size(PLATFORM_PULSE_MINIMUM_LOGICAL_EXTENT),
    );
    let platform = match WorthUiNativePlatform::prepare(profile) {
        Ok(platform) => platform,
        Err(denial) => {
            publish_event_loop_failure(&publisher);
            eprintln!("WORTH UI native platform preparation failed: {denial:?}");
            return ExitCode::from(2);
        }
    };
    match platform.run(crate::native_application::PlatformPulseApplication::new(
        launch, publisher,
    )) {
        UiNativePlatformOutcome::Closed(receipt) if receipt.terminal_census().is_zero() => {
            #[cfg(feature = "executable-world")]
            if let Err(error) = crate::native_close_evidence::write(&receipt) {
                eprintln!("WORTH UI native close evidence could not be written: {error}");
                return ExitCode::from(3);
            }
            ExitCode::SUCCESS
        }
        outcome => {
            eprintln!("WORTH UI native Platform Pulse stopped: {outcome:?}");
            ExitCode::from(3)
        }
    }
}

fn publish_event_loop_failure(publisher: &PlatformPulseObservationPublisher) {
    if let Err(publication) = publisher.native_event_loop_failure() {
        eprintln!(
            "WORTH UI platform pulse event-loop failure could not be observed: {publication:?}"
        );
    }
}
