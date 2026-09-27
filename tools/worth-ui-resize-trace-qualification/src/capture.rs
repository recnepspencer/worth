//! Samples the visible client origin of the traced window as fast as GDI allows.
//!
//! Each sample reads the primary button, the cursor, and the client extent,
//! copies the composed desktop pixels over the stamp, and brackets the copy
//! with performance-counter reads, the clock the host trace also uses. GDI
//! copies what the compositor shows; it is a pixel oracle, not a vertical-blank
//! clock, so a sighting is bounded by its sample's bracket.
//!
//! This process is aware of the system DPI only, so the window must stay on a
//! monitor at the system DPI; anywhere else Windows scales what GDI copies.

use std::io::Write;
use std::time::{Duration, Instant};

use uiautomation::screenshots::Screenshot;
use uiautomation::types::Rect;
use winsafe::{self as w, co};

use crate::logs::{capture_header, capture_line, Sample};
use crate::stamp::{self, Image, Reading};

/// The capture covers twice the stamp so a scaled stamp still shows its pitch.
const REGION: [i32; 2] = [2 * stamp::EXTENT[0] as i32, 2 * stamp::EXTENT[1] as i32];

fn failed(what: &str) -> impl Fn(w::co::ERROR) -> String + '_ {
    move |error| format!("{what}: {error}")
}

fn refresh_hz() -> Result<u32, String> {
    let mut mode = w::DEVMODE::default();
    w::EnumDisplaySettings(
        None,
        w::GmidxEnum::Enum(co::ENUM_SETTINGS::CURRENT),
        &mut mode,
    )
    .map_err(failed("reading the display mode"))?;
    Ok(mode.dmDisplayFrequency)
}

/// The Windows build and revision, as `<build>.<revision>`.
fn windows_build() -> Result<String, String> {
    let read = |name: &str| {
        w::HKEY::LOCAL_MACHINE
            .RegGetValue(
                Some(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"),
                Some(name),
                co::RRF::RT_ANY,
            )
            .map_err(failed("reading the Windows build"))
    };
    match (read("CurrentBuild")?, read("UBR")?) {
        (w::RegistryValue::Sz(build), w::RegistryValue::Dword(revision)) => {
            Ok(format!("{build}.{revision}"))
        }
        _ => Err("the Windows build is recorded in an unexpected form".to_owned()),
    }
}

fn system_dpi() -> Result<u32, String> {
    let screen = w::HWND::GetDesktopWindow()
        .GetDC()
        .map_err(failed("reading the screen"))?;
    u32::try_from(screen.GetDeviceCaps(co::GDC::LOGPIXELSX))
        .map_err(|_| "the screen reports a negative DPI".to_owned())
}

/// The key of the primary button, which the user may have swapped.
fn primary_button() -> co::VK {
    if w::GetSystemMetrics(co::SM::SWAPBUTTON) == 0 {
        co::VK::LBUTTON
    } else {
        co::VK::RBUTTON
    }
}

fn sample(window: &w::HWND, button: co::VK, dpi: u32) -> Result<Sample, String> {
    let before = w::QueryPerformanceCounter().map_err(failed("reading the counter"))?;
    let pressed = w::GetAsyncKeyState(button);
    let cursor = w::GetCursorPos().map_err(failed("reading the cursor"))?;
    if window.GetDpiForWindow() != dpi {
        return Err("the window changed DPI; keep it on one monitor while capturing".to_owned());
    }
    let client = window
        .GetClientRect()
        .map_err(failed("reading the client extent"))?;
    let origin = window
        .ClientToScreenRc(client)
        .map_err(failed("locating the client"))?;
    let extent = [client.right - client.left, client.bottom - client.top];
    let region = [extent[0].min(REGION[0]), extent[1].min(REGION[1])];
    let reading = if region[0] > 0 && region[1] > 0 {
        let shot = Screenshot::capture_rect(Rect::new(
            origin.left,
            origin.top,
            origin.left + region[0],
            origin.top + region[1],
        ))
        .map_err(|error| format!("capturing the client origin: {error}"))?
        // GDI rows arrive bottom-up; this orders them top-down.
        .to_rgba();
        stamp::read(&Image {
            pixels: shot.pixels(),
            width: shot.width() as usize,
            height: shot.height() as usize,
        })
    } else {
        Reading::Unreadable
    };
    let after = w::QueryPerformanceCounter().map_err(failed("reading the counter"))?;
    Ok(Sample {
        before,
        after,
        client: extent.map(|side| u32::try_from(side).unwrap_or(0)),
        pressed,
        cursor: [cursor.x, cursor.y],
        reading,
    })
}

/// Captures the window titled `title` for `duration`, writing the capture log
/// to `out`.
pub fn run(out: &std::path::Path, duration: Duration, title: &str) -> Result<(), String> {
    w::SetProcessDPIAware().map_err(failed("becoming DPI aware"))?;
    let window = w::HWND::FindWindow(None, Some(title))
        .map_err(failed("finding the window"))?
        .ok_or_else(|| format!("no window is titled `{title}`"))?;
    let frequency = w::QueryPerformanceFrequency().map_err(failed("reading the frequency"))?;
    let dpi = window.GetDpiForWindow();
    let system = system_dpi()?;
    if dpi != system {
        return Err(format!(
            "the window is at {dpi} DPI but the system at {system}; move it to a monitor at the system DPI"
        ));
    }
    let button = primary_button();
    let file = std::fs::File::create(out)
        .map_err(|error| format!("creating {}: {error}", out.display()))?;
    let mut log = std::io::BufWriter::new(file);
    let write = |log: &mut std::io::BufWriter<std::fs::File>, line: String| {
        writeln!(log, "{line}").map_err(|error| format!("writing {}: {error}", out.display()))
    };
    write(
        &mut log,
        capture_header(frequency, refresh_hz()?, dpi, &windows_build()?),
    )?;
    let started = Instant::now();
    let mut count = 0_usize;
    while started.elapsed() < duration && window.IsWindow() {
        write(&mut log, capture_line(&sample(&window, button, dpi)?))?;
        count += 1;
    }
    write(&mut log, format!("end {count}"))?;
    log.flush()
        .map_err(|error| format!("writing {}: {error}", out.display()))
}
