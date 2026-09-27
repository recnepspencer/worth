//! Resize qualification trace.
//!
//! When [`TRACE_PATH_ENVIRONMENT`] names a file, the native host timestamps its
//! extent path on the performance counter an outside capture also reads: each
//! extent the window reports, each extent its visible surface is replaced at,
//! each frame it submits, and each painted frame whose presentation it
//! acknowledges. Submitted frames also carry a [`stamp`] so the capture can
//! name the frame on screen. Beside the timings it records the work behind
//! them: the adapter presenting, each render target allocated, each mounted
//! frame's text work, and, as the host closes, the most of each resource it
//! retained at once.
//!
//! The file starts with `worth-ui-resize-trace 1 frequency <counts per second>`.
//! Every later line is `<counter> <event> <fields>`:
//!
//! - `observed <width> <height>`
//! - `consumed <width> <height>`
//! - `submitted <frame> <width> <height>`
//! - `accepted <frame>`
//! - `adapter <name> (<driver>)`
//! - `target <width> <height>`
//! - `text <frame> <shaped runs> <shaped scalars> <positioned glyphs>
//!   <emitted lines> <rasterized glyphs>`
//! - `peak <resource> <count>`
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

/// The host chose the graphics adapter it presents with.
pub(crate) fn adapter(name: &str, driver: &str) {
    record(format_args!("adapter {name} ({driver})"));
}

/// The host allocated a render target of `extent`.
pub(crate) fn target(extent: [u32; 2]) {
    record(format_args!("target {} {}", extent[0], extent[1]));
}

/// The most of each resource the host retained at once, as it closes.
pub(crate) fn peaks(census: crate::native::UiNativeResourceCensus) {
    for (resource, count) in census.entries() {
        record(format_args!("peak {resource} {count}"));
    }
}

/// The text one mounted frame laid out and rasterized.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiNativeResizeTraceTextWork {
    pub shaped_runs: u64,
    pub shaped_scalars: u64,
    pub positioned_glyphs: u64,
    pub emitted_lines: u64,
    pub rasterized_glyphs: u64,
}

/// Records the text work of the frame a resize trace names `frame`, the
/// attempt the frame was submitted under.
#[doc(hidden)]
pub fn trace_resize_text_work(frame: u64, work: UiNativeResizeTraceTextWork) {
    record(format_args!(
        "text {frame} {} {} {} {} {}",
        work.shaped_runs,
        work.shaped_scalars,
        work.positioned_glyphs,
        work.emitted_lines,
        work.rasterized_glyphs
    ));
}
