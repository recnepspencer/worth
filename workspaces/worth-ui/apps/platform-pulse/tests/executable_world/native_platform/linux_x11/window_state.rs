//! Window geometry and visibility actuation. Without a window manager the
//! server applies `ConfigureWindow` directly, and there is no iconic state to
//! enter: winit's X11 backend has no `UnmapNotify` handler, so an ICCCM
//! unmap/map is invisible to the product (measured 2026-09-21: no transition,
//! no successor frame). What the product does observe is `VisibilityNotify`,
//! mapped to `Occluded`, which the host treats as `Minimized` and, on
//! `Occluded(false)`, as a surface-basis successor. So the visibility
//! transition here is full occlusion by an override-redirect cover window,
//! declared as such on the observation and in the profile record.
use std::thread;
use std::time::{Duration, Instant};

use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{
    ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt as _, CreateWindowAux, EventMask,
    StackMode, Visibility, Window, WindowClass,
};
use x11rb::protocol::Event;

use crate::external_observation::{
    NativeClientAreaBounds, NativeWindowVisibilityTransitionMechanism,
    NativeWindowVisibilityTransitionObservation, ProcessBoundNativeClientAreaObservation,
};
use crate::native_platform::NativePlatformFailure;

use super::connection::{actuation_failure, X11Observation};
use super::{process_windows, LinuxX11ProcessBoundNativeClientArea};

const COVER_MARGIN: i32 = 8;

/// Raise the window so a root read at its bounds shows its pixels.
pub(super) fn expose(x11: &X11Observation, window: Window) -> Result<(), NativePlatformFailure> {
    x11.connection()
        .configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )
        .map_err(|error| NativePlatformFailure::ClientExposure(error.to_string()))?
        .check()
        .map_err(|error| NativePlatformFailure::ClientExposure(error.to_string()))
}

pub(super) fn resize(
    x11: &X11Observation,
    bound: &mut LinuxX11ProcessBoundNativeClientArea,
    client_physical_size: [u32; 2],
    deadline: Instant,
) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
    if client_physical_size.contains(&0) {
        return Err(NativePlatformFailure::WindowActuation(
            "zero client extent".to_owned(),
        ));
    }
    let width =
        u32::from(u16::try_from(client_physical_size[0]).map_err(|_| {
            NativePlatformFailure::WindowActuation("client width overflow".to_owned())
        })?);
    let height = u32::from(u16::try_from(client_physical_size[1]).map_err(|_| {
        NativePlatformFailure::WindowActuation("client height overflow".to_owned())
    })?);
    x11.connection()
        .configure_window(
            bound.window,
            &ConfigureWindowAux::new().width(width).height(height),
        )
        .map_err(actuation_failure)?
        .check()
        .map_err(actuation_failure)?;
    let observation = loop {
        let observed = current_observation(x11, bound)?;
        if [observed.bounds().width(), observed.bounds().height()] == client_physical_size {
            break observed;
        }
        if Instant::now() >= deadline {
            return Err(NativePlatformFailure::WindowStateDeadline("resized"));
        }
        thread::sleep(Duration::from_millis(20));
    };
    bound.observation = observation;
    Ok(observation)
}

pub(super) fn occlude_and_uncover(
    x11: &X11Observation,
    bound: &mut LinuxX11ProcessBoundNativeClientArea,
    deadline: Instant,
) -> Result<NativeWindowVisibilityTransitionObservation, NativePlatformFailure> {
    let connection = x11.connection();
    connection
        .change_window_attributes(
            bound.window,
            &ChangeWindowAttributesAux::new()
                .event_mask(EventMask::VISIBILITY_CHANGE | EventMask::STRUCTURE_NOTIFY),
        )
        .map_err(actuation_failure)?
        .check()
        .map_err(actuation_failure)?;
    drain_events(x11)?;
    let cover = CoverWindow::map_over(x11, bound.observation.bounds())?;
    await_visibility(
        x11,
        bound.window,
        Visibility::FULLY_OBSCURED,
        deadline,
        "minimized",
    )?;
    drop(cover);
    await_visibility(
        x11,
        bound.window,
        Visibility::UNOBSCURED,
        deadline,
        "restored",
    )?;
    let restored = current_observation(x11, bound)?;
    if restored.bounds() != bound.observation.bounds() {
        return Err(NativePlatformFailure::BoundClientAreaChanged);
    }
    bound.observation = restored;
    Ok(NativeWindowVisibilityTransitionObservation::observed(
        NativeWindowVisibilityTransitionMechanism::FullOcclusion,
        restored,
    ))
}

/// Re-derives every field the binding observed; the facade compares.
pub(super) fn current_observation(
    x11: &X11Observation,
    bound: &LinuxX11ProcessBoundNativeClientArea,
) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
    if !process_windows::window_exists(x11, bound.window)? {
        return Err(NativePlatformFailure::BoundWindowMissing);
    }
    if x11.window_process_id(bound.window)? != Some(bound.observation.process_id()) {
        return Err(NativePlatformFailure::BoundWindowOwnerChanged);
    }
    let bounds = process_windows::client_bounds(x11, bound.window)?
        .ok_or(NativePlatformFailure::BoundWindowMissing)?;
    let dpi = x11
        .qualify_dpi()
        .map_err(|_| NativePlatformFailure::BoundWindowDpiChanged)?;
    if dpi != bound.observation.dpi() {
        return Err(NativePlatformFailure::BoundWindowDpiChanged);
    }
    Ok(ProcessBoundNativeClientAreaObservation::new(
        bound.observation.process_id(),
        bound.observation.window(),
        bounds,
        dpi,
        bound.observation.window_lookup_count(),
    ))
}

/// An override-redirect window over the bound window's bounds. Dropping it
/// destroys the cover, on the success path and on every failure path alike.
struct CoverWindow<'x11> {
    x11: &'x11 X11Observation,
    window: Window,
}

impl<'x11> CoverWindow<'x11> {
    fn map_over(
        x11: &'x11 X11Observation,
        bounds: NativeClientAreaBounds,
    ) -> Result<Self, NativePlatformFailure> {
        let connection = x11.connection();
        let window = connection.generate_id().map_err(actuation_failure)?;
        let x = i16::try_from(bounds.left() - COVER_MARGIN).map_err(actuation_failure)?;
        let y = i16::try_from(bounds.top() - COVER_MARGIN).map_err(actuation_failure)?;
        let width =
            u16::try_from(bounds.width() as i32 + 2 * COVER_MARGIN).map_err(actuation_failure)?;
        let height =
            u16::try_from(bounds.height() as i32 + 2 * COVER_MARGIN).map_err(actuation_failure)?;
        connection
            .create_window(
                x11rb::COPY_DEPTH_FROM_PARENT,
                window,
                x11.root(),
                x,
                y,
                width,
                height,
                0,
                WindowClass::INPUT_OUTPUT,
                x11rb::COPY_FROM_PARENT,
                &CreateWindowAux::new()
                    .override_redirect(1)
                    .background_pixel(x11.screen().black_pixel),
            )
            .map_err(actuation_failure)?
            .check()
            .map_err(actuation_failure)?;
        let cover = Self { x11, window };
        connection
            .map_window(window)
            .map_err(actuation_failure)?
            .check()
            .map_err(actuation_failure)?;
        Ok(cover)
    }
}

impl Drop for CoverWindow<'_> {
    fn drop(&mut self) {
        let connection = self.x11.connection();
        let destroyed = connection
            .destroy_window(self.window)
            .map_err(x11rb::errors::ReplyError::from)
            .and_then(|cookie| cookie.check());
        assert!(
            destroyed.is_ok(),
            "the occlusion cover must not outlive the transition: {destroyed:?}"
        );
    }
}

fn drain_events(x11: &X11Observation) -> Result<(), NativePlatformFailure> {
    while x11
        .connection()
        .poll_for_event()
        .map_err(actuation_failure)?
        .is_some()
    {}
    Ok(())
}

/// The server's own `VisibilityNotify` for the bound window is the witness:
/// the same event winit turns into `Occluded`, read on the observer's side.
fn await_visibility(
    x11: &X11Observation,
    window: Window,
    state: Visibility,
    deadline: Instant,
    name: &'static str,
) -> Result<(), NativePlatformFailure> {
    loop {
        match x11
            .connection()
            .poll_for_event()
            .map_err(actuation_failure)?
        {
            Some(Event::VisibilityNotify(event))
                if event.window == window && event.state == state =>
            {
                return Ok(());
            }
            Some(Event::DestroyNotify(event)) if event.window == window => {
                return Err(NativePlatformFailure::BoundWindowMissing);
            }
            Some(_) => continue,
            None => {}
        }
        if Instant::now() >= deadline {
            return Err(NativePlatformFailure::WindowStateDeadline(name));
        }
        thread::sleep(Duration::from_millis(10));
    }
}
