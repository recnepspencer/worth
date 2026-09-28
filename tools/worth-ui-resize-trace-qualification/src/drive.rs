//! Runs one qualification with no one at the controls: launches the native
//! host with its resize trace, drags the window's bottom-right corner with
//! operating-system input while the capture samples, closes the window, and
//! collects how the host ended.
//!
//! The drag is real input, so the host sees the same modal sizing loop a
//! person's drag enters. It takes the mouse for its duration and puts the
//! cursor back afterwards.

use std::f64::consts::TAU;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use winsafe::{self as w, co};

/// The client extent, in logical pixels, the drag narrows to at each turn:
/// past the 800x600 a graded drag must reach, and below the 1200-pixel layout
/// breakpoint, which the drag therefore crosses in both directions.
const NARROWEST: [i32; 2] = [780, 580];
/// Seconds per narrowing and widening. A drag lasts whole periods, so it
/// ends where it began.
pub const PERIOD: f64 = 4.0;
/// Physical pixels past the client corner: inside the resize border.
const GRAB: i32 = 4;
const STEP: Duration = Duration::from_millis(8);
/// Capture before the press and after the release.
const LEAD: Duration = Duration::from_secs(1);
const SETTLE: Duration = Duration::from_secs(2);
const READY_WITHIN: Duration = Duration::from_secs(90);
const CLOSE_WITHIN: Duration = Duration::from_secs(30);

/// Where a drive left its logs and how the host ended.
pub struct Drive {
    pub host_trace: PathBuf,
    pub capture: PathBuf,
    /// The host's exit code; `None` when it had to be ended.
    pub exit: Option<i32>,
    pub stderr: String,
    /// The capture's own failure, if it had one.
    pub capture_error: Option<String>,
}

/// How far the corner has travelled inward `elapsed` seconds into the drag,
/// given how far it `reaches` at each turn. It starts and, after whole
/// periods, ends at no travel.
pub fn travel(elapsed: f64, reach: [i32; 2]) -> [i32; 2] {
    let depth = (1.0 - (TAU * elapsed / PERIOD).cos()) / 2.0;
    reach.map(|side| (f64::from(side) * depth).round() as i32)
}

fn failed(what: &str) -> impl Fn(w::co::ERROR) -> String + '_ {
    move |error| format!("{what}: {error}")
}

fn pause(duration: Duration) {
    std::thread::sleep(duration);
}

/// `requested`, rounded up to whole periods of travel.
pub fn whole_periods(requested: Duration) -> Duration {
    Duration::from_secs_f64((requested.as_secs_f64() / PERIOD).ceil().max(1.0) * PERIOD)
}

fn mouse(flags: co::MOUSEEVENTF, at: [i32; 2]) -> Result<(), String> {
    let origin = [
        w::GetSystemMetrics(co::SM::XVIRTUALSCREEN),
        w::GetSystemMetrics(co::SM::YVIRTUALSCREEN),
    ];
    let extent = [
        w::GetSystemMetrics(co::SM::CXVIRTUALSCREEN),
        w::GetSystemMetrics(co::SM::CYVIRTUALSCREEN),
    ];
    // Windows maps a normalized coordinate back as `normal * extent / 65536`,
    // truncating, so rounding up here lands on exactly the intended pixel.
    let normal = |axis: usize| {
        let (offset, extent) = (
            i64::from(at[axis] - origin[axis]),
            i64::from(extent[axis].max(1)),
        );
        i32::try_from((offset * 65_536 + extent - 1) / extent).unwrap_or(i32::MAX)
    };
    w::SendInput(&[w::HwKbMouse::Mouse(w::MOUSEINPUT {
        dx: normal(0),
        dy: normal(1),
        dwFlags: flags
            | co::MOUSEEVENTF::MOVE
            | co::MOUSEEVENTF::ABSOLUTE
            | co::MOUSEEVENTF::VIRTUALDESK,
        ..Default::default()
    })])
    .map(|_| ())
    .map_err(failed("sending mouse input"))
}

/// Presses, drags, and releases the primary button at the window's corner.
fn drag(window: &w::HWND, duration: Duration) -> Result<(), String> {
    let client = window
        .GetClientRect()
        .map_err(failed("reading the client extent"))?;
    let screen = window
        .ClientToScreenRc(client)
        .map_err(failed("locating the client"))?;
    let scale = |side: i32| side * i32::try_from(window.GetDpiForWindow()).unwrap_or(96) / 96;
    let reach = [
        (client.right - scale(NARROWEST[0])).max(0),
        (client.bottom - scale(NARROWEST[1])).max(0),
    ];
    let corner = [screen.right + GRAB, screen.bottom + GRAB];
    let (down, up) = if crate::capture::primary_button() == co::VK::LBUTTON {
        (co::MOUSEEVENTF::LEFTDOWN, co::MOUSEEVENTF::LEFTUP)
    } else {
        (co::MOUSEEVENTF::RIGHTDOWN, co::MOUSEEVENTF::RIGHTUP)
    };
    window.SetForegroundWindow();
    mouse(co::MOUSEEVENTF::MOVE, corner)?;
    pause(Duration::from_millis(300));
    // Press only on the host's own border, never on a window over it.
    let under = w::HWND::WindowFromPoint(w::POINT::with(corner[0], corner[1]))
        .and_then(|under| under.GetAncestor(co::GA::ROOT));
    if under.as_ref() != Some(window) {
        return Err("another window covers the host's corner".to_owned());
    }
    mouse(down, corner)?;
    pause(Duration::from_millis(200));
    let begun = Instant::now();
    let moved = (|| {
        while begun.elapsed() < duration {
            let inward = travel(begun.elapsed().as_secs_f64(), reach);
            mouse(
                co::MOUSEEVENTF::MOVE,
                [corner[0] - inward[0], corner[1] - inward[1]],
            )?;
            pause(STEP);
        }
        mouse(co::MOUSEEVENTF::MOVE, corner)
    })();
    // Release whatever happened, so a failure never leaves the button held.
    pause(Duration::from_millis(200));
    let released = mouse(up, corner);
    moved.and(released)
}

/// Asks the window to close as a person would, only once it is foremost.
/// The keys go to whichever window is foremost when they arrive, so the
/// check leaves a moment in which another window could take the close.
fn close(window: &w::HWND) -> bool {
    window.SetForegroundWindow();
    pause(Duration::from_millis(200));
    if w::HWND::GetForegroundWindow().as_ref() != Some(window) {
        return false;
    }
    let key = |key: co::VK, flags: co::KEYEVENTF| {
        w::HwKbMouse::Kb(w::KEYBDINPUT {
            wVk: key,
            dwFlags: flags,
            ..Default::default()
        })
    };
    w::SendInput(&[
        key(co::VK::MENU, co::KEYEVENTF::NoValue),
        key(co::VK::F4, co::KEYEVENTF::NoValue),
        key(co::VK::F4, co::KEYEVENTF::KEYUP),
        key(co::VK::MENU, co::KEYEVENTF::KEYUP),
    ])
    .is_ok()
}

fn wait_for_exit(child: &mut Child, within: Duration) -> Result<Option<i32>, String> {
    let begun = Instant::now();
    while begun.elapsed() < within {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return Ok(status.code());
        }
        pause(Duration::from_millis(100));
    }
    child
        .kill()
        .map_err(|error| format!("ending the host: {error}"))?;
    child.wait().map_err(|error| error.to_string())?;
    Ok(None)
}

/// Waits until the host has presented a frame and shows its window. The
/// capture finds the window by `title`, so a window of that title the host
/// does not own ends the drive before anything is pressed.
fn wait_until_ready(child: &mut Child, trace: &Path, title: &str) -> Result<w::HWND, String> {
    let begun = Instant::now();
    while begun.elapsed() < READY_WITHIN {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err("the host exited before presenting".to_owned());
        }
        let presented = std::fs::read_to_string(trace)
            .is_ok_and(|text| text.lines().any(|line| line.contains(" accepted ")));
        if presented {
            if let Some(window) =
                w::HWND::FindWindow(None, Some(title)).map_err(failed("finding the window"))?
            {
                if window.GetWindowThreadProcessId().1 != child.id() {
                    return Err(format!(
                        "a window titled \"{title}\" that the host does not own is open"
                    ));
                }
                return Ok(window);
            }
        }
        pause(Duration::from_millis(100));
    }
    Err(format!(
        "the host presented no frame within {} s",
        READY_WITHIN.as_secs()
    ))
}

/// Launches `host`, drags its window for `duration`, and closes it, writing
/// its logs beside `prefix`. A `duration` of whole periods, as
/// [`whole_periods`] gives, ends the drag where it began.
pub fn run(host: &Path, prefix: &str, duration: Duration, title: &str) -> Result<Drive, String> {
    w::SetProcessDPIAware().map_err(failed("becoming DPI aware"))?;
    let path = |suffix: &str| PathBuf::from(format!("{prefix}.{suffix}"));
    let (host_trace, capture, stderr) = (path("host"), path("capture"), path("stderr"));
    let create = |path: &Path| {
        std::fs::File::create(path).map_err(|error| format!("creating {}: {error}", path.display()))
    };
    let _ = std::fs::remove_file(&host_trace);
    let mut child = Command::new(host)
        .env("WORTH_UI_RESIZE_TRACE", &host_trace)
        .stdout(Stdio::from(create(&path("stdout"))?))
        .stderr(Stdio::from(create(&stderr)?))
        .spawn()
        .map_err(|error| format!("launching {}: {error}", host.display()))?;
    let cursor = w::GetCursorPos().map_err(failed("reading the cursor"))?;
    let driven = wait_until_ready(&mut child, &host_trace, title).and_then(|window| {
        let (log, name) = (capture.clone(), title.to_owned());
        let capturing = std::thread::spawn(move || {
            crate::capture::run(
                &log,
                LEAD + duration + SETTLE,
                &name,
                crate::logs::Grip::BottomRight,
            )
        });
        pause(LEAD);
        let dragged = drag(&window, duration);
        let captured = capturing
            .join()
            .map_err(|_| "the capture panicked".to_owned())?;
        dragged?;
        Ok((window, captured.err()))
    });
    let closed = driven.as_ref().is_ok_and(|(window, _)| close(window));
    let _ = w::SetCursorPos(cursor.x, cursor.y);
    let exit = wait_for_exit(&mut child, if closed { CLOSE_WITHIN } else { SETTLE })?;
    let stderr = std::fs::read_to_string(&stderr).unwrap_or_default();
    let (_, capture_error) = driven.map_err(|error| format!("{error}\n{stderr}"))?;
    Ok(Drive {
        host_trace,
        capture,
        exit,
        stderr,
        capture_error,
    })
}

#[cfg(test)]
#[path = "tests/drive.rs"]
mod tests;
