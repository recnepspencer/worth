//! Resize qualification trace.
//!
//! When [`TRACE_PATH_ENVIRONMENT`] names a file, the native host timestamps its
//! extent path on the performance counter an outside capture also reads: the
//! least extent the window allows, each extent the window reports, each extent
//! its visible surface is replaced at, each frame it submits, and each painted
//! frame whose presentation it acknowledges. Submitted frames also carry a
//! [`stamp`] so the capture can name the frame on screen. Beside the timings it
//! records the work behind them: the adapter presenting, each render target
//! allocated, each size the swapchain is configured at, each mounted frame's
//! text work, the span of each named stage of a frame, and, as the host closes,
//! the most of each resource it retained at once.
//!
//! The file starts with `worth-ui-resize-trace 1 frequency <counts per second>`.
//! Every later line is `<counter> <event> <fields>`:
//!
//! - `minimum <width> <height>`, when the window opens and again at each scale change
//! - `observed <width> <height>`
//! - `consumed <width> <height>`
//! - `submitted <frame> <width> <height>`
//! - `accepted <frame>`
//! - `adapter <name> (<driver>)`, the driver only when the adapter reports one
//! - `target <width> <height>`
//! - `swapchain <width> <height>`
//! - `text <frame> <shaped runs> <shaped scalars> <positioned glyphs>
//!   <emitted lines> <rasterized glyphs>`
//! - `peak <resource> <count>`
//! - `stage <stage> <start counter>`, as each named stage of a frame ends
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
    // One write per line: the file is unbuffered, and writing the formatted
    // pieces one by one would cost a system call each on the frame path.
    let line = format!("{counter} {event}\n");
    let Ok(mut file) = trace.lock() else {
        return;
    };
    let failed = file
        .as_mut()
        .is_some_and(|open| open.write_all(line.as_bytes()).is_err());
    if failed {
        *file = None;
    }
}

/// The window allows no client extent smaller than `extent`, from its opening
/// or from a change of scale.
pub(crate) fn minimum(extent: [u32; 2]) {
    record(format_args!("minimum {} {}", extent[0], extent[1]));
}

/// The window reported a new client extent.
pub(crate) fn observed(size: [u32; 2]) {
    record(format_args!("observed {} {}", size[0], size[1]));
}

/// The host replaced its visible surface at a client extent.
pub(crate) fn consumed(size: [u32; 2]) {
    record(format_args!("consumed {} {}", size[0], size[1]));
}

/// The host handed a frame of `extent` to the surface for display, and the
/// work counted since the previous submission is charged to it.
pub(crate) fn submitted(frame: u64, extent: [u32; 2]) {
    record(format_args!(
        "submitted {frame} {} {}",
        extent[0], extent[1]
    ));
    work(frame);
}

/// Records the presentation work counted since the previous submission,
/// attempts that never submitted included, as `frame`'s: glyph records per
/// [`worth_ui_host_contract::UiPresentationWorkStage::ALL`] stage, then
/// digested bytes, map inserts, and allocations (`-` when uncounted). Taking
/// the counts even when no trace is written starts each frame afresh.
///
/// Like every trace line, this one is formatted into a single allocation,
/// which the next frame's count includes along with the frame's other lines.
fn work(frame: u64) {
    let work = worth_ui_host_contract::take_presentation_work();
    if enabled() {
        record(format_args!("work {frame} {}", WorkFields(work)));
    }
}

/// The fields of a `work` line, written without allocating.
struct WorkFields(worth_ui_host_contract::UiPresentationWorkCounts);

impl std::fmt::Display for WorkFields {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let work = self.0;
        for stage in worth_ui_host_contract::UiPresentationWorkStage::ALL {
            write!(formatter, "{} ", work.glyphs(stage))?;
        }
        write!(
            formatter,
            "{} {} ",
            work.digested_bytes(),
            work.map_inserts()
        )?;
        match work.allocations() {
            Some(allocations) => write!(formatter, "{allocations}"),
            None => formatter.write_str("-"),
        }
    }
}

/// The host acknowledged the presentation of a frame it painted.
pub(crate) fn accepted(frame: u64) {
    record(format_args!("accepted {frame}"));
}

/// The host chose the graphics adapter it presents with.
pub(crate) fn adapter(name: &str, driver: &str) {
    if driver.is_empty() {
        record(format_args!("adapter {name}"));
    } else {
        record(format_args!("adapter {name} ({driver})"));
    }
}

/// The host allocated a render target of `extent`.
pub(crate) fn target(extent: [u32; 2]) {
    record(format_args!("target {} {}", extent[0], extent[1]));
}

/// The host configured its swapchain at `extent`.
pub(crate) fn swapchain(extent: [u32; 2]) {
    record(format_args!("swapchain {} {}", extent[0], extent[1]));
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

/// A named stage of the work behind one frame.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeResizeTraceStage {
    /// The whole frame, from its framework turn to its presentation.
    Frame,
    /// Settling the host measurements mounted views asked for.
    Settle,
    /// Projecting the mounted content into a frame.
    Projection,
    /// Lowering the frame's appearance with its overlays.
    Appearance,
    /// Admitting pending asynchronous text work.
    Async,
    /// Publishing and validating the frame's native layout.
    Layout,
    /// Completing the frame's presentation after the host draws it.
    Completion,
    /// Admitting the frame's text into the glyph atlas.
    Atlas,
    /// Drawing the frame's presentation delta.
    Draw,
    /// Acquiring the surface texture the frame is drawn into.
    Acquire,
    /// Encoding the frame's draw commands.
    Encode,
    /// Submitting the frame's commands to the device.
    Submit,
    /// Replacing the visible surface at a new extent.
    Surface,
}

impl UiNativeResizeTraceStage {
    const fn name(self) -> &'static str {
        match self {
            Self::Frame => "frame",
            Self::Settle => "settle",
            Self::Projection => "projection",
            Self::Appearance => "appearance",
            Self::Async => "async",
            Self::Layout => "layout",
            Self::Completion => "completion",
            Self::Atlas => "atlas",
            Self::Draw => "draw",
            Self::Acquire => "acquire",
            Self::Encode => "encode",
            Self::Submit => "submit",
            Self::Surface => "surface",
        }
    }
}

/// A stage of a frame in progress, recorded with its span when dropped.
#[doc(hidden)]
#[must_use = "a stage is recorded when its span is dropped"]
pub struct UiNativeResizeTraceSpan {
    stage: UiNativeResizeTraceStage,
    start: Option<i64>,
}

/// Opens the span of `stage`, recorded as it drops; without a trace, it
/// records nothing and reads no counter.
#[doc(hidden)]
pub fn trace_resize_stage(stage: UiNativeResizeTraceStage) -> UiNativeResizeTraceSpan {
    let start = if enabled() { clock::counter() } else { None };
    UiNativeResizeTraceSpan { stage, start }
}

impl Drop for UiNativeResizeTraceSpan {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            record(format_args!("stage {} {start}", self.stage.name()));
        }
    }
}
