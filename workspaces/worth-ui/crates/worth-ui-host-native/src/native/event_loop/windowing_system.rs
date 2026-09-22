//! Binds the winit builder to the windowing system the qualified profile names.
//!
//! winit picks its Linux backend at run time from `WAYLAND_DISPLAY` and
//! `DISPLAY`; the profile is picked at compile time. Left uncoupled, an
//! X11-profile build on a Wayland session would configure `Opaque` — which
//! Wayland also offers — and certify a Wayland surface under a record that
//! says X11. The event loop therefore forces the backend from
//! [`WORTH_UI_NATIVE_WINDOWING_SYSTEM`] before it is built, and applies the
//! thread posture through that same backend's extension trait.
//!
//! The X11 backend has several system preconditions (`libX11`, `libxcb`,
//! `libxkbcommon-x11` and its dependency `libxcb-xkb`). winit reports all but
//! the last two as `EventLoopError`; it loads `libxkbcommon-x11` with `dlopen`
//! and panics when that library or its dependency is absent. Only those two
//! are located ahead of time, so an unavailable backend is a typed denial
//! before any effect — the posture every profile declares as
//! `unsupported_mode = "typed-denial-before-effects-no-fallback"`.

#[cfg(target_os = "linux")]
mod x11_keyboard_library;

use winit::event_loop::EventLoopBuilder;

use super::{UiNativeEventLoopRunDenial, UiNativeEventLoopThreadPosture};
#[cfg(any(target_os = "windows", target_os = "linux"))]
use crate::native_profile::{UiNativeWindowingSystem, WORTH_UI_NATIVE_WINDOWING_SYSTEM};

/// Forces the builder onto the qualified windowing system and applies the
/// thread posture through its extension trait, or denies before the event
/// loop exists.
///
/// Both Linux traits write the same builder field; dispatching on the profile
/// keeps the call attributed to the backend that will actually run.
#[cfg(target_os = "linux")]
pub(super) fn force_qualified<T>(
    builder: &mut EventLoopBuilder<T>,
    thread_posture: UiNativeEventLoopThreadPosture,
) -> Result<(), UiNativeEventLoopRunDenial> {
    use winit::platform::wayland::EventLoopBuilderExtWayland;
    use winit::platform::x11::EventLoopBuilderExtX11;

    match WORTH_UI_NATIVE_WINDOWING_SYSTEM {
        UiNativeWindowingSystem::Wayland => {
            builder.with_wayland();
            EventLoopBuilderExtWayland::with_any_thread(builder, thread_posture.any_thread());
            Ok(())
        }
        UiNativeWindowingSystem::X11 => {
            x11_keyboard_library::locate(&x11_keyboard_library::loader_search_directories())
                .map_err(|_| UiNativeEventLoopRunDenial::WindowingSystemUnavailable)?;
            builder.with_x11();
            EventLoopBuilderExtX11::with_any_thread(builder, thread_posture.any_thread());
            Ok(())
        }
        UiNativeWindowingSystem::Win32 => {
            Err(UiNativeEventLoopRunDenial::WindowingSystemUnavailable)
        }
    }
}

/// Windows offers one windowing system, which winit drives without forcing;
/// the thread posture is applied through the Windows extension trait.
#[cfg(target_os = "windows")]
pub(super) fn force_qualified<T>(
    builder: &mut EventLoopBuilder<T>,
    thread_posture: UiNativeEventLoopThreadPosture,
) -> Result<(), UiNativeEventLoopRunDenial> {
    match WORTH_UI_NATIVE_WINDOWING_SYSTEM {
        UiNativeWindowingSystem::Win32 => {
            thread_posture.configure(builder);
            Ok(())
        }
        UiNativeWindowingSystem::Wayland | UiNativeWindowingSystem::X11 => {
            Err(UiNativeEventLoopRunDenial::WindowingSystemUnavailable)
        }
    }
}

/// No windowing system is qualified for this target, so the compiled
/// reference profile cannot be driven here.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub(super) fn force_qualified<T>(
    _builder: &mut EventLoopBuilder<T>,
    _thread_posture: UiNativeEventLoopThreadPosture,
) -> Result<(), UiNativeEventLoopRunDenial> {
    Err(UiNativeEventLoopRunDenial::WindowingSystemUnavailable)
}
