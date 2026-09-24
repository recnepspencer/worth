use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseSemanticFocusCause,
};

use crate::external_observation::{
    NativeClientPixelCapture, NativeClientPixelPoint, NativeKeyboardCommand,
    PlatformPulseLifecycleStream,
};
use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};
use crate::native_platform::{
    CertifiedNativePlatform, NativePlatformContract, NativePlatformFailure,
};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

#[test]
fn timeline_selector_opens_from_a_native_pointer_activation() {
    let mut installation = IsolatedPulseInstallation::install(CanonicalPlatformPulse::checked_in())
        .expect("install the authored dashboard");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .expect("resolve the product executable")
        .launch(
            installation.source_root(),
            installation.source_root(),
            installation.source_root(),
        )
        .expect("launch the authored product");
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        let envelope = launch.lifecycle.next(deadline).expect("first frame event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::FirstFramePublished(_) => break,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before timeline input: {failure:?}")
            }
            _ => {}
        }
    }
    let platform = CertifiedNativePlatform::certified().expect("native desktop available");
    let mut client = platform
        .bind_process_client_area(launch.process.id(), deadline)
        .expect("bind the product client area");
    let closed = await_capture(&platform, &client, deadline);
    open_timeline_selector(&platform, &client, &mut launch.lifecycle, &closed, deadline);
    let chart = NativeClientPixelPoint::interior(
        &closed,
        560 * closed.width() / 1536,
        420 * closed.height() / 1024,
        1,
    )
    .expect("chart target is within the client area");
    unsettled_pointer_activation(&platform, &client, chart);
    await_timeline_dismissal(&mut launch.lifecycle, deadline);
    await_timeline_closed_pixels(&platform, &client, &closed, deadline, "chart");
    platform
        .minimize_and_restore_bound_client_area(&mut client, deadline)
        .expect("restore the same native client area");
    let restored = await_capture(&platform, &client, deadline);
    open_timeline_selector(
        &platform,
        &client,
        &mut launch.lifecycle,
        &restored,
        deadline,
    );
    platform
        .deliver_keyboard_command(&client, NativeKeyboardCommand::Escape)
        .expect("dismiss the restored timeline menu");
    await_timeline_dismissal(&mut launch.lifecycle, deadline);
    await_timeline_closed_pixels(&platform, &client, &restored, deadline, "Escape");
    let closed_again = await_capture(&platform, &client, deadline);
    open_timeline_selector(
        &platform,
        &client,
        &mut launch.lifecycle,
        &closed_again,
        deadline,
    );
    select_timeline_option(&platform, &client, &closed_again);
    await_timeline_closed_pixels(&platform, &client, &closed_again, deadline, "option");
    for _ in 0..3 {
        let before = await_capture(&platform, &client, deadline);
        open_timeline_selector(&platform, &client, &mut launch.lifecycle, &before, deadline);
        select_timeline_option(&platform, &client, &before);
        await_timeline_closed_pixels(&platform, &client, &before, deadline, "repeated option");
    }
    for _ in 0..3 {
        let before = await_capture(&platform, &client, deadline);
        activate_timeline_selector(&platform, &client, &mut launch.lifecycle, &before, deadline);
        unsettled_pointer_activation(&platform, &client, chart);
        await_timeline_dismissal(&mut launch.lifecycle, deadline);
        await_timeline_closed_pixels(&platform, &client, &before, deadline, "rapid chart");
    }
    platform
        .request_normal_close(&client)
        .expect("close product");
    loop {
        let envelope = launch.lifecycle.next(deadline).expect("shutdown event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::ShutdownCompleted(_) => break,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during timeline cleanup: {failure:?}")
            }
            _ => {}
        }
    }
    SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline).expect("product exits");
    installation.close().expect("release authored source");
}

fn select_timeline_option(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    capture: &NativeClientPixelCapture,
) {
    let period_option = NativeClientPixelPoint::interior(
        capture,
        1000 * capture.width() / 1536,
        400 * capture.height() / 1024,
        1,
    )
    .expect("timeline option remains in the native client area");
    unsettled_pointer_activation(platform, client, period_option);
}

fn await_timeline_dismissal(lifecycle: &mut PlatformPulseLifecycleStream, deadline: Instant) {
    let mut dismissed = false;
    let mut focus_restored = false;
    while !dismissed || !focus_restored {
        let envelope = lifecycle
            .next(deadline)
            .expect("timeline dismissal lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::PortalDismissed(_) => dismissed = true,
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus)
                if focus.cause() == PlatformPulseSemanticFocusCause::PortalRestoration =>
            {
                focus_restored = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during timeline dismissal: {failure:?}")
            }
            _ => {}
        }
    }
}

fn open_timeline_selector(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    lifecycle: &mut PlatformPulseLifecycleStream,
    closed: &NativeClientPixelCapture,
    deadline: Instant,
) {
    activate_timeline_selector(platform, client, lifecycle, closed, deadline);
    loop {
        let open = await_capture(platform, client, deadline);
        let changed = changed_menu_pixels(&closed, &open);
        if changed >= 24 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timeline menu did not reach native pixels"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn activate_timeline_selector(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    lifecycle: &mut PlatformPulseLifecycleStream,
    closed: &NativeClientPixelCapture,
    deadline: Instant,
) {
    let selector = NativeClientPixelPoint::interior(
        &closed,
        995 * closed.width() / 1536,
        300 * closed.height() / 1024,
        1,
    )
    .expect("timeline selector is within the client area");
    platform
        .deliver_pointer_activation(client, selector)
        .expect("open timeline selector");
    loop {
        let envelope = lifecycle.next(deadline).expect("timeline lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus)
                if focus.cause() == PlatformPulseSemanticFocusCause::PortalInitial =>
            {
                break
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed after timeline input: {failure:?}")
            }
            _ => {}
        }
    }
}

fn await_capture(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    deadline: Instant,
) -> NativeClientPixelCapture {
    loop {
        match platform.capture_client_area(client) {
            Ok(capture) => return capture,
            Err(NativePlatformFailure::ClientCapture(_)) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(failure) => panic!("native client pixels did not settle: {failure:?}"),
        }
    }
}

fn unsettled_pointer_activation(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    point: NativeClientPixelPoint,
) {
    #[cfg(windows)]
    {
        // Preserve hover/press overlap: no pixel-stability wait, and release
        // in a later native turn instead of batching down/up in one SendInput.
        let prepared = platform
            .prepare_wheel_input(client, point)
            .expect("prepare pointer");
        std::thread::sleep(Duration::from_millis(30));
        let (held, _) = prepared.press_primary().expect("press pointer");
        std::thread::sleep(Duration::from_millis(65));
        held.release().expect("release pointer");
    }
    #[cfg(not(windows))]
    platform
        .deliver_pointer_activation(client, point)
        .expect("activate pointer");
}

fn await_timeline_closed_pixels(
    platform: &CertifiedNativePlatform,
    client: &crate::native_platform::CertifiedProcessBoundNativeClientArea,
    closed: &NativeClientPixelCapture,
    deadline: Instant,
    dismissal: &str,
) {
    loop {
        let capture = await_capture(platform, client, deadline);
        let changed = changed_menu_pixels(closed, &capture);
        if changed < 24 {
            return;
        }
        if Instant::now() >= deadline {
            panic!("{dismissal} dismissal never removed the menu from native pixels: {changed} changed pixels");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn changed_menu_pixels(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
) -> usize {
    assert_eq!(
        [before.width(), before.height()],
        [after.width(), after.height()]
    );
    let width = before.width();
    // The trigger's hover/focus fill changes independently of the menu.
    // Compare only the actual popup body below that trigger.
    let left = 930 * width / 1536;
    let right = 1130 * width / 1536;
    let top = 360 * before.height() / 1024;
    let bottom = 520 * before.height() / 1024;
    (top..bottom)
        .flat_map(|row| (left..right).map(move |column| (row, column)))
        .filter(|(row, column)| {
            let index = ((*row * width + *column) * 4) as usize;
            before.rgba()[index..index + 3] != after.rgba()[index..index + 3]
        })
        .count()
}
