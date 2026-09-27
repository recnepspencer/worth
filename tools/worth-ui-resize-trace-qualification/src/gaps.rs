//! The stretches between accepted frames becoming visible, and the part of each
//! during which the window was owed a frame.
//!
//! A gap ends at the first sighting of a new accepted frame. The gap still open
//! at release ends at the first new frame seen after release, or at the end of
//! the capture if none is. The window is owed a frame from the earliest of:
//!
//! - the gap's start, if the window's latest observed extent differs from the
//!   frame then shown;
//! - the first extent other than the shown frame's that the window observed
//!   inside the gap;
//! - the first cursor movement inside the gap with the button held, if the
//!   window observed such an extent inside the gap.
//!
//! The cursor is read by the capture, not the host, so a host that stops
//! handling window messages cannot hide the demand it owes. Cursor movement
//! with no new extent observed, as against a minimum size, is not demand, nor
//! is a repeated report of the extent shown. A gap owing nothing is idle.
//!
//! A stall that ends with a new frame at the unchanged extent, before the
//! window reports the extents queued behind it, splits into an idle gap and an
//! active one owed only from the report, so the grade understates that stall.

use crate::analysis::{Clock, Sighting};
use crate::logs::{HostEvent, HostKind, Sample};

/// One accepted frame's first sighting, and the stretch before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub start: f64,
    pub end: f64,
    /// From the moment the window was owed a frame, if it was.
    pub active: Option<f64>,
}

/// When the window was first owed a frame between `since`, when `shown`
/// became visible, and `end`.
fn owed(
    events: &[HostEvent],
    samples: &[Sample],
    shown: &Sighting,
    since: i64,
    end: i64,
) -> Option<i64> {
    let latest = events.iter().rev().find_map(|event| match event.kind {
        HostKind::Observed(extent) if event.counter <= since => Some(extent),
        _ => None,
    });
    let unmet = latest.is_some_and(|extent| extent != shown.extent);
    let observed = events.iter().find_map(|event| match event.kind {
        HostKind::Observed(extent)
            if event.counter > since && event.counter < end && extent != shown.extent =>
        {
            Some(event.counter)
        }
        _ => None,
    });
    let moved = observed.and_then(|_| {
        samples.windows(2).find_map(|pair| {
            let sample = pair[1];
            (sample.before > since
                && sample.before < end
                && sample.pressed
                && sample.cursor != pair[0].cursor)
                .then_some(sample.before)
        })
    });
    [unmet.then_some(since), observed, moved]
        .into_iter()
        .flatten()
        .min()
}

/// The gaps from the frame visible at the press, `first`, until the first new
/// frame after release, `released`.
pub fn gaps(
    samples: &[Sample],
    sightings: &[Option<Option<Sighting>>],
    events: &[HostEvent],
    [first, released]: [usize; 2],
    clock: &Clock,
) -> Vec<Gap> {
    let mut gaps = Vec::new();
    let mut previous: Option<(Sighting, i64)> = None;
    let mut closed = false;
    let record = |gaps: &mut Vec<Gap>, shown: &Sighting, since: i64, end: i64| {
        gaps.push(Gap {
            start: clock.ms(since),
            end: clock.ms(end),
            active: owed(events, samples, shown, since, end)
                .map(|owed| clock.ms(end) - clock.ms(owed)),
        });
    };
    for (index, (sample, found)) in samples.iter().zip(sightings).enumerate() {
        let Some(Some(frame)) = found else { continue };
        if !frame.accepted || previous.is_some_and(|(shown, _)| shown.frame == frame.frame) {
            continue;
        }
        if index > first {
            if let Some((shown, since)) = previous {
                record(&mut gaps, &shown, since, sample.after);
            }
        }
        previous = Some((*frame, sample.after));
        if index >= released {
            closed = true;
            break;
        }
    }
    if let (false, Some((shown, since)), Some(last)) = (closed, previous, samples.last()) {
        if last.after > since {
            record(&mut gaps, &shown, since, last.after);
        }
    }
    gaps
}

#[cfg(test)]
#[path = "tests/gaps.rs"]
mod tests;
