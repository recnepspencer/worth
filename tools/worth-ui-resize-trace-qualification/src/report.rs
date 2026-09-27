//! Grades an analysis against the 3.16.2 live-resize thresholds and lists the
//! raw intervals behind each grade.

use std::fmt::Write;

use crate::analysis::{Analysis, MOVING_PAUSE_MS};
use crate::coverage::{BREAKPOINT_WIDTH, LARGE_EXTENT, SMALL_EXTENT};
use crate::logs::CaptureLog;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    /// Only grades that depend on the capture's sightings failed, and the
    /// capture sampled too coarsely, or ended too abruptly, to trust them.
    Inconclusive,
}

/// Nearest-rank percentile.
fn percentile(values: &[f64], fraction: f64) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (fraction * sorted.len() as f64).ceil() as usize;
    sorted.get(rank.max(1) - 1).copied()
}

fn shown(value: Option<f64>) -> String {
    match value {
        None => "none".to_owned(),
        Some(value) if value.is_infinite() => "never".to_owned(),
        Some(value) => format!("{value:.1} ms"),
    }
}

/// What a grade rests on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Basis {
    /// The host trace and the run's conditions.
    Host,
    /// What the capture happened to see.
    Sightings,
    /// The capture's own resolution, which decides whether sighting grades
    /// can be trusted.
    Capture,
}

#[derive(Default)]
struct Grades {
    text: String,
    host_failed: bool,
    sightings_failed: bool,
}

impl Grades {
    fn check(&mut self, basis: Basis, name: &str, measured: String, limit: &str, pass: bool) {
        match basis {
            Basis::Host => self.host_failed |= !pass,
            Basis::Sightings => self.sightings_failed |= !pass,
            Basis::Capture => {}
        }
        let grade = if pass { "pass" } else { "FAIL" };
        let _ = writeln!(self.text, "  {grade:4}  {name}: {measured} (limit {limit})");
    }
}

fn grade_run(grades: &mut Grades, analysis: &Analysis, capture: &CaptureLog) {
    grades.check(
        Basis::Host,
        "display refresh",
        format!("{} Hz", capture.refresh_hz),
        "60 Hz",
        (59..=61).contains(&capture.refresh_hz),
    );
    grades.check(
        Basis::Host,
        "moving drag",
        format!(
            "{:.0} ms of cursor movement, pauses over {MOVING_PAUSE_MS:.0} ms excluded, within {:.0} ms held",
            analysis.moving_ms, analysis.drag_ms
        ),
        ">= 10000 ms",
        analysis.moving_ms >= 10_000.0,
    );
    let coverage = analysis.coverage;
    grades.check(
        Basis::Host,
        "logical extent coverage",
        format!(
            "{}x{} to {}x{}, {} reversals, crossed width {BREAKPOINT_WIDTH} {} times narrowing and {} widening",
            coverage.smallest[0],
            coverage.smallest[1],
            coverage.largest[0],
            coverage.largest[1],
            coverage.reversals,
            coverage.narrowing,
            coverage.widening
        ),
        &format!(
            "{}x{} or less, {}x{} or more, 2 or more reversals, the breakpoint crossed both ways",
            SMALL_EXTENT[0], SMALL_EXTENT[1], LARGE_EXTENT[0], LARGE_EXTENT[1]
        ),
        coverage.reached_small
            && coverage.reached_large
            && coverage.reversals >= 2
            && coverage.narrowing >= 1
            && coverage.widening >= 1,
    );
    grades.check(
        Basis::Host,
        "stamps naming no submitted frame",
        analysis.unknown.to_string(),
        "0",
        analysis.unknown == 0,
    );
}

fn grade_timing(grades: &mut Grades, analysis: &Analysis) {
    let mut latencies: Vec<f64> = analysis
        .latencies
        .iter()
        .map(|latency| latency.visible - latency.observed)
        .collect();
    latencies.extend(analysis.never_presented.iter().map(|_| f64::INFINITY));
    let p95_latency = percentile(&latencies, 0.95);
    grades.check(
        Basis::Sightings,
        "p95 size event to first matching accepted frame",
        format!(
            "{} over {} extents: {} seen, {} never presented and counted as misses; {} accepted but unseen are excluded; {} timed from the consume",
            shown(p95_latency),
            latencies.len(),
            analysis.latencies.len(),
            analysis.never_presented.len(),
            analysis.unseen,
            analysis.consume_starts
        ),
        "50 ms",
        p95_latency.is_some_and(|value| value <= 50.0),
    );
    grades.check(
        Basis::Sightings,
        "last consumed extent never seen",
        format!("{:?}", analysis.missing),
        "none",
        analysis.missing.is_none(),
    );
    let active: Vec<f64> = analysis.gaps.iter().filter_map(|gap| gap.active).collect();
    let idle = analysis.gaps.len() - active.len();
    let p95_gap = percentile(&active, 0.95);
    let p99_gap = percentile(&active, 0.99);
    grades.check(
        Basis::Sightings,
        "p95 accepted visible-frame gap",
        format!(
            "{} over {} active gaps, {idle} idle",
            shown(p95_gap),
            active.len()
        ),
        "25 ms",
        p95_gap.is_some_and(|value| value <= 25.0),
    );
    grades.check(
        Basis::Sightings,
        "p99 accepted visible-frame gap",
        shown(p99_gap),
        "50 ms",
        p99_gap.is_some_and(|value| value <= 50.0),
    );
    let longest = percentile(&active, 1.0);
    grades.check(
        Basis::Sightings,
        "longest active gap",
        shown(longest),
        "100 ms",
        longest.is_some_and(|value| value <= 100.0),
    );
    grades.check(
        Basis::Sightings,
        "final exact extent after release",
        format!(
            "{} at {}x{}",
            shown(analysis.final_ms),
            analysis.final_extent[0],
            analysis.final_extent[1]
        ),
        "100 ms",
        analysis.final_ms.is_some_and(|value| value <= 100.0),
    );
}

fn raw(text: &mut String, analysis: &Analysis) {
    let _ = writeln!(
        text,
        "not graded: {:.1} ms stretched, {:.1} ms showing another extent, {:.1} ms unreadable, {} frames seen but never accepted, {} returns to an older frame, {} submissions at a superseded extent",
        analysis.stretched_ms,
        analysis.mismatched_ms,
        analysis.unreadable_ms,
        analysis.unaccepted,
        analysis.regressions,
        analysis.stale_submissions
    );
    let _ = writeln!(text, "never presented: {:?}", analysis.never_presented);
    let _ = writeln!(
        text,
        "latencies (extent, observed ms, visible ms, latency ms):"
    );
    for latency in &analysis.latencies {
        let _ = writeln!(
            text,
            "  {}x{} {:.3} {:.3} {:.3}",
            latency.extent[0],
            latency.extent[1],
            latency.observed,
            latency.visible,
            latency.visible - latency.observed
        );
    }
    let _ = writeln!(text, "gaps (start ms, end ms, raw ms, active ms):");
    for gap in &analysis.gaps {
        let active = gap
            .active
            .map_or_else(|| "idle".to_owned(), |active| format!("{active:.3}"));
        let _ = writeln!(
            text,
            "  {:.3} {:.3} {:.3} {active}",
            gap.start,
            gap.end,
            gap.end - gap.start
        );
    }
}

pub fn render(analysis: &Analysis, capture: &CaptureLog) -> (String, Verdict) {
    let mut text = String::new();
    let _ = writeln!(
        text,
        "clock: QueryPerformanceCounter at {} counts per second, shared by host and capture",
        capture.frequency
    );
    let _ = writeln!(
        text,
        "capture: GDI copy of the composed desktop at the client origin, bracketed by the counter"
    );
    let _ = writeln!(
        text,
        "display: {} Hz, {} DPI",
        capture.refresh_hz, capture.dpi
    );
    let mut grades = Grades::default();
    grade_run(&mut grades, analysis, capture);
    grade_timing(&mut grades, analysis);
    let intervals = &analysis.sample_intervals;
    let coarse = percentile(intervals, 0.99);
    let resolved = capture.complete && coarse.is_some_and(|value| value <= 25.0);
    grades.check(
        Basis::Capture,
        "capture p99 sample interval",
        format!(
            "{}, median {}, longest {}, {} samples{}",
            shown(coarse),
            shown(percentile(intervals, 0.5)),
            shown(percentile(intervals, 1.0)),
            intervals.len(),
            if capture.complete {
                ""
            } else {
                ", capture not closed"
            }
        ),
        "25 ms and a closed capture",
        resolved,
    );
    let verdict = if grades.host_failed {
        Verdict::Fail
    } else if !resolved {
        Verdict::Inconclusive
    } else if grades.sightings_failed {
        Verdict::Fail
    } else {
        Verdict::Pass
    };
    let _ = writeln!(text, "verdict: {verdict:?}");
    text.push_str(&grades.text);
    raw(&mut text, analysis);
    (text, verdict)
}

#[cfg(test)]
#[path = "tests/report.rs"]
mod tests;
