use uiautomation::inputs::Mouse;
use uiautomation::types::{Handle, Point};
use uiautomation::UIAutomation;
use winsafe::{co, HwKbMouse, HWND, KEYBDINPUT, MOUSEINPUT};

use crate::external_observation::{
    NativeClientPixelPoint, NativeInputDeliveryObservation, NativeInputProbeKind,
    NativeKeyboardCommand, ProcessBoundNativeClientAreaObservation,
};

use super::NativePlatformFailure;

pub(super) fn deliver(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    kind: NativeInputProbeKind,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    let screen_x = bounds
        .left()
        .checked_add_unsigned(bounds.width() / 2)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let screen_y = bounds
        .top()
        .checked_add_unsigned(bounds.height() / 2)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    deliver_at(
        window,
        observed,
        kind,
        (screen_x, screen_y),
        None,
        NativeKeyboardInput::Single(co::VK::CHAR_A),
    )
}

pub(super) fn deliver_keyboard_command(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    command: NativeKeyboardCommand,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    let point = (
        bounds.left().saturating_add_unsigned(bounds.width() / 2),
        bounds.top().saturating_add_unsigned(bounds.height() / 2),
    );
    let input = match command {
        NativeKeyboardCommand::Escape => NativeKeyboardInput::Single(co::VK::ESCAPE),
        NativeKeyboardCommand::PrimaryShiftP => NativeKeyboardInput::PrimaryShiftP,
    };
    deliver_at(
        window,
        observed,
        NativeInputProbeKind::Keyboard,
        point,
        None,
        input,
    )
}

pub(super) fn deliver_pointer(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    point: NativeClientPixelPoint,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    if point.capture_extent() != (bounds.width(), bounds.height()) {
        return Err(NativePlatformFailure::InputDelivery(
            "pointer point was adjudicated from a different client capture extent".to_owned(),
        ));
    }
    let (client_x, client_y) = point.coordinates();
    let screen_x = bounds
        .left()
        .checked_add_unsigned(client_x)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let screen_y = bounds
        .top()
        .checked_add_unsigned(client_y)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    deliver_at(
        window,
        observed,
        NativeInputProbeKind::Pointer,
        (screen_x, screen_y),
        Some(point.landing_tolerance()),
        NativeKeyboardInput::Single(co::VK::CHAR_A),
    )
}

pub(super) fn deliver_wheel_deltas(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
) -> Result<(), NativePlatformFailure> {
    let bounds = observed.bounds();
    let half_width = i32::try_from(bounds.width() / 2)
        .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let half_height = i32::try_from(bounds.height() / 2)
        .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let screen_point = (
        bounds
            .left()
            .checked_add(half_width)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        bounds
            .top()
            .checked_add(half_height)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
    );
    let automation = UIAutomation::new().map_err(input_failure)?;
    automation
        .element_from_handle(Handle::from(window.ptr() as isize))
        .and_then(|element| element.set_focus())
        .map_err(input_failure)?;
    super::input_environment::qualify_pointer_world(window, observed.process_id(), screen_point)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    move_pointer_to(screen_point)?;
    super::pointer_target::require_before_effect(window, screen_point)?;
    let delivered = winsafe::SendInput(&[
        HwKbMouse::Mouse(MOUSEINPUT {
            mouseData: 120,
            dwFlags: co::MOUSEEVENTF::WHEEL,
            ..Default::default()
        }),
        HwKbMouse::Mouse(MOUSEINPUT {
            mouseData: 120,
            dwFlags: co::MOUSEEVENTF::HWHEEL,
            ..Default::default()
        }),
    ])
    .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
    if delivered != 2 {
        return Err(NativePlatformFailure::InputDelivery(format!(
            "SendInput delivered {delivered} of 2 wheel events"
        )));
    }
    Ok(())
}

fn deliver_at(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    kind: NativeInputProbeKind,
    screen_point: (i32, i32),
    pointer_tolerance: Option<u32>,
    keyboard_input: NativeKeyboardInput,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    let (screen_x, screen_y) = screen_point;
    let automation = UIAutomation::new().map_err(input_failure)?;
    let element = automation
        .element_from_handle(Handle::from(window.ptr() as isize))
        .map_err(input_failure)?;
    element.set_focus().map_err(|error| {
        NativePlatformFailure::InputDelivery(format!(
            "focus process-bound automation element: {error}"
        ))
    })?;
    if kind == NativeInputProbeKind::Pointer {
        super::input_environment::qualify_pointer_world(
            window,
            observed.process_id(),
            (screen_x, screen_y),
        )
        .map_err(NativePlatformFailure::InputEnvironment)?;
    }
    let delivered_event_count = match kind {
        NativeInputProbeKind::Pointer => {
            prime_pointer_motion(window, (screen_x, screen_y))?;
            super::pointer_visual_settlement::await_client_stability(observed)?;
            super::pointer_target::require_before_effect(window, (screen_x, screen_y))?;
            let delivered = winsafe::SendInput(&[
                HwKbMouse::Mouse(MOUSEINPUT {
                    dwFlags: co::MOUSEEVENTF::LEFTDOWN,
                    ..Default::default()
                }),
                HwKbMouse::Mouse(MOUSEINPUT {
                    dwFlags: co::MOUSEEVENTF::LEFTUP,
                    ..Default::default()
                }),
            ])
            .map_err(|error| post_effect_failure(kind, 0, error.to_string()))?;
            require_complete_delivery(kind, delivered, 2, "pointer")?;
            delivered
        }
        NativeInputProbeKind::Keyboard => {
            super::input_environment::qualify_keyboard_world(window, observed.process_id())
                .map_err(NativePlatformFailure::InputEnvironment)?;
            if !element.has_keyboard_focus().map_err(input_failure)? {
                return Err(NativePlatformFailure::InputEnvironment(
                    super::input_environment::WindowsInputEnvironmentDenial::
                        KeyboardFocusUnavailable {
                            target_window: window.ptr() as usize,
                        },
                ));
            }
            let expected = keyboard_input.expected_event_count();
            let delivered = deliver_keyboard_events(keyboard_input, kind)?;
            require_complete_delivery(kind, delivered, expected, "keyboard")?;
            let retained_focus = element.has_keyboard_focus().map_err(|error| {
                post_effect_failure(
                    kind,
                    delivered,
                    format!("observe process-bound keyboard focus after delivery: {error}"),
                )
            })?;
            if !retained_focus {
                return Err(post_effect_failure(
                    kind,
                    delivered,
                    "process-bound automation element lost keyboard focus",
                ));
            }
            delivered
        }
    };
    let delivered_point = match kind {
        NativeInputProbeKind::Pointer => {
            super::pointer_target::require_after_effect(
                window,
                (screen_x, screen_y),
                kind,
                delivered_event_count,
            )?;
            let delivered_point = Mouse::get_cursor_pos().map_err(|error| {
                post_effect_failure(
                    kind,
                    delivered_event_count,
                    format!("observe cursor position after delivery: {error}"),
                )
            })?;
            if pointer_tolerance.is_some_and(|tolerance| {
                delivered_point.get_x().abs_diff(screen_x) > tolerance
                    || delivered_point.get_y().abs_diff(screen_y) > tolerance
            }) {
                return Err(post_effect_failure(
                    kind,
                    delivered_event_count,
                    format!(
                        "native pointer landed at ({}, {}) instead of ({screen_x}, {screen_y})",
                        delivered_point.get_x(),
                        delivered_point.get_y()
                    ),
                ));
            }
            (delivered_point.get_x(), delivered_point.get_y())
        }
        NativeInputProbeKind::Keyboard => (screen_x, screen_y),
    };
    if delivered_point.0 < bounds.left()
        || delivered_point.0 >= bounds.right()
        || delivered_point.1 < bounds.top()
        || delivered_point.1 >= bounds.bottom()
    {
        return Err(post_effect_failure(
            kind,
            delivered_event_count,
            "native input target was outside the process-bound client area",
        ));
    }
    Ok(NativeInputDeliveryObservation::for_client(
        kind,
        observed,
        delivered_point,
        delivered_event_count,
    ))
}

fn require_complete_delivery(
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
    expected_event_count: u32,
    family: &'static str,
) -> Result<(), NativePlatformFailure> {
    if delivered_event_count != expected_event_count {
        return Err(post_effect_failure(
            kind,
            delivered_event_count,
            format!(
                "SendInput delivered {delivered_event_count} of {expected_event_count} {family} events"
            ),
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum NativeKeyboardInput {
    Single(co::VK),
    PrimaryShiftP,
}

impl NativeKeyboardInput {
    const fn expected_event_count(self) -> u32 {
        match self {
            Self::Single(_) => 2,
            Self::PrimaryShiftP => 6,
        }
    }
}

fn deliver_keyboard_events(
    input: NativeKeyboardInput,
    kind: NativeInputProbeKind,
) -> Result<u32, NativePlatformFailure> {
    match input {
        NativeKeyboardInput::Single(key) => {
            send_keyboard_batch(kind, &[key_down(key), key_up(key)])
        }
        NativeKeyboardInput::PrimaryShiftP => {
            let pressed =
                send_keyboard_batch(kind, &[key_down(co::VK::CONTROL), key_down(co::VK::SHIFT)])?;
            std::thread::sleep(std::time::Duration::from_millis(10));
            let invoked =
                send_keyboard_batch(kind, &[key_down(co::VK::CHAR_P), key_up(co::VK::CHAR_P)])?;
            std::thread::sleep(std::time::Duration::from_millis(10));
            let released =
                send_keyboard_batch(kind, &[key_up(co::VK::SHIFT), key_up(co::VK::CONTROL)])?;
            Ok(pressed + invoked + released)
        }
    }
}

fn send_keyboard_batch(
    kind: NativeInputProbeKind,
    events: &[HwKbMouse],
) -> Result<u32, NativePlatformFailure> {
    winsafe::SendInput(events).map_err(|error| post_effect_failure(kind, 0, error.to_string()))
}

fn key_down(key: co::VK) -> HwKbMouse {
    HwKbMouse::Kb(KEYBDINPUT {
        wVk: key,
        ..Default::default()
    })
}

fn key_up(key: co::VK) -> HwKbMouse {
    HwKbMouse::Kb(KEYBDINPUT {
        wVk: key,
        dwFlags: co::KEYEVENTF::KEYUP,
        ..Default::default()
    })
}

pub(super) fn post_effect_failure(
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
    detail: impl Into<String>,
) -> NativePlatformFailure {
    NativePlatformFailure::InputDeliveryIndeterminate {
        kind,
        delivered_event_count,
        detail: detail.into(),
    }
}

fn move_pointer_to(screen_point: (i32, i32)) -> Result<(), NativePlatformFailure> {
    Mouse::set_cursor_pos(&Point::new(screen_point.0, screen_point.1)).map_err(input_failure)
}

fn prime_pointer_motion(
    window: &HWND,
    screen_point: (i32, i32),
) -> Result<(), NativePlatformFailure> {
    let adjacent = (
        screen_point
            .0
            .checked_sub(1)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        screen_point.1,
    );
    super::input_environment::actuate_and_observe_cursor(adjacent)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    super::pointer_target::require_before_effect(window, adjacent)?;
    super::input_environment::actuate_and_observe_cursor(screen_point)
        .map_err(NativePlatformFailure::InputEnvironment)
}

fn input_failure(error: uiautomation::Error) -> NativePlatformFailure {
    NativePlatformFailure::InputDelivery(error.to_string())
}
