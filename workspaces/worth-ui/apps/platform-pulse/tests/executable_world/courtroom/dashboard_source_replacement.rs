use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservation;

use crate::adjudication::dashboard_visual_oracle as oracle;
use crate::external_observation::NativeClientPixelCapture;
use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};
use crate::native_platform::{CertifiedNativePlatform, NativePlatformContract};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};
use crate::source_delta::{
    CanonicalBlueRecoverySourceDelta, GreenPulseSourceDelta, MalformedPulseSourceDelta,
};

#[test]
fn imported_signal_source_rebind_preserves_and_recovers_native_pixels() {
    let canonical = CanonicalPlatformPulse::checked_in();
    let mut installation = IsolatedPulseInstallation::install(canonical)
        .expect("install exact current dashboard source");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .expect("resolve product executable")
        .launch(
            installation.source_root(),
            installation.source_root(),
            installation.source_root(),
        )
        .expect("launch current dashboard");
    let deadline = Instant::now() + Duration::from_secs(40);
    let mut first = None;
    let mut pending = false;
    while first.is_none() || !pending {
        match launch
            .lifecycle
            .next(deadline)
            .expect("first publication")
            .outcome()
        {
            PlatformPulseLifecycleObservation::FirstFramePublished(frame) => first = Some(*frame),
            PlatformPulseLifecycleObservation::QueryProjectionPublished(_) => pending = true,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before source journey: {failure:?}")
            }
            _ => {}
        }
    }
    let first = first.expect("first frame published");
    let platform = CertifiedNativePlatform::certified().expect("native desktop available");
    let client = platform
        .bind_process_client_area(launch.process.id(), deadline)
        .expect("bind product client area");
    await_signal(&platform, &client, oracle::POSITIVE_RGB, deadline);

    GreenPulseSourceDelta::from_checked_in(canonical)
        .expect("derive the unique imported signal edit")
        .apply(&installation)
        .expect("atomically replace navigation source");
    // Each source transition has its own bounded observation window. The
    // initial launch/capture work must not consume a later transition's budget.
    let deadline = Instant::now() + Duration::from_secs(40);
    let successor = loop {
        match launch
            .lifecycle
            .next(deadline)
            .expect("source successor")
            .outcome()
        {
            PlatformPulseLifecycleObservation::RebindPublished(replacement) => break *replacement,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed while rebinding signal: {failure:?}")
            }
            _ => {}
        }
    };
    assert_ne!(successor.active_generation(), first.generation());
    assert!(successor.actual_native_effect_count() > 0);
    await_signal(&platform, &client, oracle::CAUTION_RGB, deadline);

    MalformedPulseSourceDelta::stable()
        .apply(&installation)
        .expect("atomically publish malformed imported source");
    let deadline = Instant::now() + Duration::from_secs(40);
    let preserved = loop {
        match launch
            .lifecycle
            .next(deadline)
            .expect("preserved predecessor")
            .outcome()
        {
            PlatformPulseLifecycleObservation::RebindDeniedPreserving(preserved) => {
                break *preserved
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product terminated instead of preserving predecessor: {failure:?}")
            }
            _ => {}
        }
    };
    assert_eq!(preserved.active_generation(), successor.active_generation());
    await_signal(&platform, &client, oracle::CAUTION_RGB, deadline);

    CanonicalBlueRecoverySourceDelta::exact(canonical)
        .apply(&installation)
        .expect("atomically restore exact imported source");
    let deadline = Instant::now() + Duration::from_secs(40);
    let recovered = loop {
        match launch
            .lifecycle
            .next(deadline)
            .expect("source recovery")
            .outcome()
        {
            PlatformPulseLifecycleObservation::RebindPublished(replacement) => break *replacement,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during exact source recovery: {failure:?}")
            }
            _ => {}
        }
    };
    assert_eq!(
        recovered.source().final_package_digest(),
        first.source().final_package_digest()
    );
    await_signal(&platform, &client, oracle::POSITIVE_RGB, deadline);

    platform
        .request_normal_close(&client)
        .expect("close product window");
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        match launch
            .lifecycle
            .next(deadline)
            .expect("shutdown publication")
            .outcome()
        {
            PlatformPulseLifecycleObservation::ShutdownCompleted(_) => break,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during shutdown: {failure:?}")
            }
            _ => {}
        }
    }
    SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("product exits after normal close");
    platform
        .verify_process_window_released(launch.process.id())
        .expect("native window released");
    installation.close().expect("remove isolated source");
}

fn await_signal(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    expected: [u8; 3],
    deadline: Instant,
) {
    loop {
        let pixels = platform
            .capture_client_area(client)
            .expect("capture signal pixels");
        if signal_matches(&pixels, expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "signal did not reach expected native pixels"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn signal_matches(pixels: &NativeClientPixelCapture, expected: [u8; 3]) -> bool {
    let x = oracle::SOURCE_SIGNAL_POINT[0] * pixels.width() / oracle::LOGICAL_EXTENT[0];
    let y = oracle::SOURCE_SIGNAL_POINT[1] * pixels.height() / oracle::LOGICAL_EXTENT[1];
    (y - 1..=y + 1).all(|row| {
        (x - 1..=x + 1).all(|column| {
            let offset = ((row * pixels.width() + column) * 4) as usize;
            pixels.rgba().get(offset..offset + 3).is_some_and(|pixel| {
                pixel.iter().zip(expected).all(|(actual, expected)| {
                    actual.abs_diff(expected) <= oracle::CHANNEL_TOLERANCE
                })
            })
        })
    })
}
