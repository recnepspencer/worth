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
//! - the first cursor movement inside the gap with the button held that moves
//!   the dragged corner, if the window observed such an extent inside the gap.
//!
//! The cursor is read by the capture, not the host, so a host that stops
//! handling window messages cannot hide the demand it owes. When the capture
//! knows the drag holds the bottom-right corner, as a driven drag does, the
//! corner follows the cursor from where it was pressed, but not below the
//! least extent the host traced for the window: movement there cannot resize
//! the window, so the drag holding the window at its minimum owes nothing
//! until the cursor comes back. When the grip is unknown, as in a person's
//! drag of any edge, or the host traced no minimum, all held movement counts.
//! Cursor movement with no new extent observed is not demand, nor is a
//! repeated report of the extent shown. A gap owing nothing is idle.
//!
//! A stall that ends with a new frame at the unchanged extent, before the
//! window reports the extents queued behind it, splits into an idle gap and an
//! active one owed only from the report, so the grade understates that stall.

use crate::analysis::{Clock, Sighting};
use crate::logs::{Grip, HostEvent, HostKind, Sample};

/// One accepted frame's first sighting, and the stretch before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub start: f64,
    pub end: f64,
    /// From the moment the window was owed a frame, if it was.
    pub active: Option<f64>,
}

/// The drag the gaps are measured over.
pub struct Drag<'a> {
    pub samples: &'a [Sample],
    pub events: &'a [HostEvent],
    /// The least client extent the window allows, if the host traced one.
    pub minimum: Option<[u32; 2]>,
    /// What the drag holds. Only a known corner is held at the minimum.
    pub grip: Grip,
    /// The first held sample, and the first sample after release.
    pub span: [usize; 2],
}

impl Drag<'_> {
    /// Where the drag puts the client extent at `sample`, from where the
    /// press grabbed it, as though it held the bottom-right corner. Held
    /// there, the extent stops at the traced minimum; with the grip unknown,
    /// it follows every movement of the cursor.
    fn implied(&self, sample: &Sample) -> [i64; 2] {
        let press = &self.samples[self.span[0]];
        let minimum = self.minimum.filter(|_| self.grip == Grip::BottomRight);
        std::array::from_fn(|axis| {
            let moved = i64::from(sample.cursor[axis]) - i64::from(press.cursor[axis]);
            let free = i64::from(press.client[axis]) + moved;
            minimum.map_or(free, |minimum| free.max(i64::from(minimum[axis])))
        })
    }
}

/// When the window was first owed a frame between `since`, when `shown`
/// became visible, and `end`.
fn owed(drag: &Drag<'_>, shown: &Sighting, since: i64, end: i64) -> Option<i64> {
    let (events, samples) = (drag.events, drag.samples);
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
                && drag.implied(&sample) != drag.implied(&pair[0]))
            .then_some(sample.before)
        })
    });
    [unmet.then_some(since), observed, moved]
        .into_iter()
        .flatten()
        .min()
}

/// The gaps from the frame visible at the press until the first new frame
/// after release.
pub fn gaps(drag: &Drag<'_>, sightings: &[Option<Option<Sighting>>], clock: &Clock) -> Vec<Gap> {
    let (samples, [first, released]) = (drag.samples, drag.span);
    let mut gaps = Vec::new();
    let mut previous: Option<(Sighting, i64)> = None;
    let mut closed = false;
    let record = |gaps: &mut Vec<Gap>, shown: &Sighting, since: i64, end: i64| {
        gaps.push(Gap {
            start: clock.ms(since),
            end: clock.ms(end),
            active: owed(drag, shown, since, end).map(|owed| clock.ms(end) - clock.ms(owed)),
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
