//! Resize qualification trace.
//!
//! When [`TRACE_PATH_ENVIRONMENT`] names a file, the native host timestamps its
//! extent path on the performance counter an outside capture also reads: each
//! extent the window reports, each extent its visible surface is replaced at,
//! each frame it submits, and each painted frame whose presentation it
//! acknowledges. Submitted frames also carry a [`stamp`] so the capture can
//! name the frame on screen.
//!
//! The file starts with `worth-ui-resize-trace 1 frequency <counts per second>`.
//! Every later line is `<counter> <event> <fields>`:
//!
//! - `observed <width> <height>`
//! - `consumed <width> <height>`
//! - `submitted <frame> <width> <height>`
//! - `accepted <frame>`
//!
//! A write failure ends the trace, so a trace is complete up to its last line.
//! Without the variable, or where no shared counter exists, nothing is traced.

use std::io::Write;
use std::sync::{Mutex, OnceLock};

mod clock;
mod stamp;

pub(crate) use stamp::{stamp_texels, STAMP_EXTENT};

/// Names the file a resize qualification trace is written to.
pub(crate) const TRACE_PATH_ENVIRONMENT: &str = "WORTH_UI_RESIZE_TRACE";

static TRACE: OnceLock<Option<Mutex<Option<std::fs::File>>>> = OnceLock::new();

fn trace() -> Option<&'static Mutex<Option<std::fs::File>>> {
    TRACE.get_or_init(open).as_ref()
}

fn open() -> Option<Mutex<Option<std::fs::File>>> {
    let path = std::env::var_os(TRACE_PATH_ENVIRONMENT)?;
    let frequency = clock::frequency()?;
    let mut file = std::fs::File::create(path).ok()?;
    writeln!(file, "worth-ui-resize-trace 1 frequency {frequency}").ok()?;
    Some(Mutex::new(Some(file)))
}

/// Reports whether this process writes a resize qualification trace.
pub(crate) fn enabled() -> bool {
    trace().is_some()
}

fn record(event: std::fmt::Arguments<'_>) {
    let Some(trace) = trace() else {
        return;
    };
    let Some(counter) = clock::counter() else {
        return;
    };
    let Ok(mut file) = trace.lock() else {
        return;
    };
    let failed = file
        .as_mut()
        .is_some_and(|open| writeln!(open, "{counter} {event}").is_err());
    if failed {
        *file = None;
    }
}

/// The window reported a new client extent.
pub(crate) fn observed(size: [u32; 2]) {
    record(format_args!("observed {} {}", size[0], size[1]));
}

/// The host replaced its visible surface at a client extent.
pub(crate) fn consumed(size: [u32; 2]) {
    record(format_args!("consumed {} {}", size[0], size[1]));
}

/// The host handed a frame of `extent` to the surface for display.
pub(crate) fn submitted(frame: u64, extent: [u32; 2]) {
    record(format_args!(
        "submitted {frame} {} {}",
        extent[0], extent[1]
    ));
}

/// The host acknowledged the presentation of a frame it painted.
pub(crate) fn accepted(frame: u64) {
    record(format_args!("accepted {frame}"));
}
