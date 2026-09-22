use super::{
    capture_failure, NativePlatformFailure, WindowsProcessBoundNativeClientArea, MAX_FRAME_BYTES,
};
use crate::external_observation::NativeClientAreaBounds;
use winsafe::{co, HMONITOR, HWND, RECT};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CaptureCrop {
    pub(super) monitor_handle: usize,
    pub(super) frame_extent: [u32; 2],
    pub(super) edges: [u32; 4],
    outer: NativeClientAreaBounds,
}

impl CaptureCrop {
    pub(super) fn qualify(
        bound: &WindowsProcessBoundNativeClientArea,
        strip: [u32; 4],
    ) -> Result<Self, NativePlatformFailure> {
        let (monitor_handle, bounds) = monitor_bounds(bound)?;
        let mut crop =
            Self::from_bounds(bounds, bound.observation.bounds(), strip).ok_or_else(|| {
                capture_failure("strip is empty, oversized, or outside one monitor/client")
            })?;
        crop.monitor_handle = monitor_handle;
        Ok(crop)
    }

    pub(super) fn from_bounds(
        outer: NativeClientAreaBounds,
        client: NativeClientAreaBounds,
        [x, y, width, height]: [u32; 4],
    ) -> Option<Self> {
        if width == 0
            || height == 0
            || x.checked_add(width)? > client.width()
            || y.checked_add(height)? > client.height()
            || client.left() < outer.left()
            || client.top() < outer.top()
            || client.right() > outer.right()
            || client.bottom() > outer.bottom()
            || (width as usize)
                .checked_mul(height as usize)?
                .checked_mul(4)?
                > MAX_FRAME_BYTES
        {
            return None;
        }
        let left = u32::try_from(client.left().checked_sub(outer.left())?)
            .ok()?
            .checked_add(x)?;
        let top = u32::try_from(client.top().checked_sub(outer.top())?)
            .ok()?
            .checked_add(y)?;
        Some(Self {
            monitor_handle: 0,
            frame_extent: [outer.width(), outer.height()],
            edges: [
                left,
                top,
                left.checked_add(width)?,
                top.checked_add(height)?,
            ],
            outer,
        })
    }

    pub(super) fn require_current(
        &self,
        bound: &WindowsProcessBoundNativeClientArea,
    ) -> Result<(), NativePlatformFailure> {
        if monitor_bounds(bound)? != (self.monitor_handle, self.outer) {
            return Err(capture_failure("capture monitor changed during trace"));
        }
        if HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
            return Err(capture_failure("bound capture window lost foreground"));
        }
        self.require_unoccluded_strip(bound)?;
        Ok(())
    }

    fn require_unoccluded_strip(
        &self,
        bound: &WindowsProcessBoundNativeClientArea,
    ) -> Result<(), NativePlatformFailure> {
        let [left, top, right, bottom] = self.edges;
        let rect = RECT {
            left: self.outer.left() + left as i32,
            top: self.outer.top() + top as i32,
            right: self.outer.left() + right as i32,
            bottom: self.outer.top() + bottom as i32,
        };
        let pointer = winsafe::GetCursorPos().map_err(capture_failure)?;
        if pointer.x >= rect.left - 64
            && pointer.x < rect.right + 64
            && pointer.y >= rect.top - 64
            && pointer.y < rect.bottom + 64
        {
            return Err(capture_failure("pointer overlaps the measured pixel strip"));
        }
        // GW_HWNDPREV explicitly names the window above this one in Z-order;
        // EnumWindows enumeration order is not that contract. Bound the walk
        // and reject disappearing/repeated HWNDs instead of proving visibility
        // from an incomplete or unstable list.
        let mut visited = std::collections::HashSet::new();
        visited.insert(bound.window.ptr() as usize);
        let mut cursor = previous_window(&bound.window)?;
        while let Some(window) = cursor {
            if visited.len() >= 4_096
                || !visited.insert(window.ptr() as usize)
                || !window.IsWindow()
            {
                return Err(capture_failure(
                    "unstable or excessive capture Z-order chain",
                ));
            }
            if window.IsWindowVisible() && !window.IsIconic() {
                let other = window.GetWindowRect().map_err(capture_failure)?;
                if other.left < rect.right
                    && other.right > rect.left
                    && other.top < rect.bottom
                    && other.bottom > rect.top
                {
                    return Err(capture_failure(
                        "foreign window may cover the measured strip",
                    ));
                }
            }
            cursor = previous_window(&window)?;
        }
        Ok(())
    }
}

fn previous_window(window: &HWND) -> Result<Option<HWND>, NativePlatformFailure> {
    if !window.IsWindow() {
        return Err(capture_failure("capture Z-order window disappeared"));
    }
    winsafe::SetLastError(co::ERROR::SUCCESS);
    match window.GetWindow(co::GW::HWNDPREV) {
        Ok(previous) => Ok(Some(previous)),
        Err(co::ERROR::SUCCESS) => Ok(None),
        Err(error) => Err(capture_failure(error)),
    }
}

fn monitor_bounds(
    bound: &WindowsProcessBoundNativeClientArea,
) -> Result<(usize, NativeClientAreaBounds), NativePlatformFailure> {
    let client = bound.observation.bounds();
    let monitor = HMONITOR::MonitorFromRect(
        RECT {
            left: client.left(),
            top: client.top(),
            right: client.right(),
            bottom: client.bottom(),
        },
        co::MONITOR::DEFAULTTONULL,
    );
    let rect = monitor.GetMonitorInfo().map_err(capture_failure)?.rcMonitor;
    let bounds = NativeClientAreaBounds::new(rect.left, rect.top, rect.right, rect.bottom)
        .ok_or_else(|| capture_failure("empty capture monitor"))?;
    Ok((monitor.ptr() as usize, bounds))
}
