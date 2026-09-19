use std::thread;
use std::time::{Duration, Instant};

use winsafe::{co, HwndPlace, POINT, SIZE};

use crate::external_observation::{
    NativeWindowVisibilityTransitionObservation, ProcessBoundNativeClientAreaObservation,
};
use crate::native_platform::NativePlatformFailure;

use super::WindowsProcessBoundNativeClientArea;

pub(super) fn resize(
    bound: &mut WindowsProcessBoundNativeClientArea,
    client_physical_size: [u32; 2],
    deadline: Instant,
) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
    if client_physical_size.contains(&0) {
        return Err(NativePlatformFailure::WindowActuation(
            "zero client extent".to_owned(),
        ));
    }
    let client = bound.observation.bounds();
    let outer = bound
        .window
        .GetWindowRect()
        .map_err(|error| NativePlatformFailure::WindowActuation(error.to_string()))?;
    let nonclient_width = (outer.right - outer.left) - client.width() as i32;
    let nonclient_height = (outer.bottom - outer.top) - client.height() as i32;
    let width = i32::try_from(client_physical_size[0])
        .ok()
        .and_then(|value| value.checked_add(nonclient_width))
        .ok_or_else(|| NativePlatformFailure::WindowActuation("client width overflow".into()))?;
    let height = i32::try_from(client_physical_size[1])
        .ok()
        .and_then(|value| value.checked_add(nonclient_height))
        .ok_or_else(|| NativePlatformFailure::WindowActuation("client height overflow".into()))?;
    bound
        .window
        .SetWindowPos(
            HwndPlace::None,
            POINT::default(),
            SIZE {
                cx: width,
                cy: height,
            },
            co::SWP::NOMOVE | co::SWP::NOZORDER | co::SWP::NOACTIVATE,
        )
        .map_err(|error| NativePlatformFailure::WindowActuation(error.to_string()))?;
    let observation = await_client_size(bound, client_physical_size, deadline)?;
    bound.observation = observation;
    Ok(observation)
}

pub(super) fn minimize_and_restore(
    bound: &mut WindowsProcessBoundNativeClientArea,
    deadline: Instant,
) -> Result<NativeWindowVisibilityTransitionObservation, NativePlatformFailure> {
    bound.window.ShowWindow(co::SW::MINIMIZE);
    await_state(bound, deadline, true, "minimized")?;
    bound.window.ShowWindow(co::SW::RESTORE);
    await_state(bound, deadline, false, "restored")?;
    winsafe::DwmFlush()
        .map_err(|error| NativePlatformFailure::WindowActuation(error.to_string()))?;
    let restored = current_observation(bound)?;
    if restored.bounds() != bound.observation.bounds() {
        return Err(NativePlatformFailure::BoundClientAreaChanged);
    }
    bound.observation = restored;
    Ok(NativeWindowVisibilityTransitionObservation::observed(
        restored,
    ))
}

fn await_client_size(
    bound: &WindowsProcessBoundNativeClientArea,
    expected: [u32; 2],
    deadline: Instant,
) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
    loop {
        let observed = current_observation(bound)?;
        if [observed.bounds().width(), observed.bounds().height()] == expected {
            return Ok(observed);
        }
        if Instant::now() >= deadline {
            return Err(NativePlatformFailure::WindowStateDeadline("resized"));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn await_state(
    bound: &WindowsProcessBoundNativeClientArea,
    deadline: Instant,
    iconic: bool,
    name: &'static str,
) -> Result<(), NativePlatformFailure> {
    loop {
        if !bound.window.IsWindow() {
            return Err(NativePlatformFailure::BoundWindowMissing);
        }
        if bound.window.IsIconic() == iconic {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(NativePlatformFailure::WindowStateDeadline(name));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn current_observation(
    bound: &WindowsProcessBoundNativeClientArea,
) -> Result<ProcessBoundNativeClientAreaObservation, NativePlatformFailure> {
    if !bound.window.IsWindow() {
        return Err(NativePlatformFailure::BoundWindowMissing);
    }
    let (_, process_id) = bound.window.GetWindowThreadProcessId();
    if process_id != bound.observation.process_id() {
        return Err(NativePlatformFailure::BoundWindowOwnerChanged);
    }
    let dpi = bound.window.GetDpiForWindow();
    if dpi != bound.observation.dpi() {
        return Err(NativePlatformFailure::BoundWindowDpiChanged);
    }
    Ok(ProcessBoundNativeClientAreaObservation::new(
        process_id,
        bound.observation.window(),
        super::capture_region::client_bounds(&bound.window)?,
        dpi,
        bound.observation.window_lookup_count(),
    ))
}
