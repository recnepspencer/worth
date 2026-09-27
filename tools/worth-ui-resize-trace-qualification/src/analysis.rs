//! Correlates a host resize trace with a capture of the stamped client origin.
//!
//! Times are milliseconds after the press that starts the drag, the longest run
//! of samples with the primary button held. A frame is visible from the first
//! sample that reads it, at that sample's closing counter, so every visible time
//! is an upper bound within one sample interval.
//!
//! Each extent the host consumed during the drag ends one of four ways:
//!
//! - a latency: an accepted frame drawn at it, submitted before the next
//!   consume, was seen;
//! - unseen: such a frame was accepted, but no sample caught it;
//! - never presented: no frame drawn at it was accepted before the next
//!   consume, so preparing it was wasted;
//! - missing: the last consumed extent, never seen.

use std::collections::BTreeSet;

use crate::coverage::{coverage, Coverage};
use crate::gaps::{gaps, Gap};
use crate::logs::{CaptureLog, HostEvent, HostKind, HostTrace, Sample};
use crate::stamp::Reading;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Latency {
    pub extent: [u32; 2],
    /// The window first reported the extent, in its latest run of reports of it.
    pub observed: f64,
    /// The first accepted frame drawn at that extent was visible. The window
    /// may already be at another extent by then; that time counts as
    /// mismatched.
    pub visible: f64,
}

#[derive(Debug, Default, PartialEq)]
pub struct Analysis {
    pub press: i64,
    /// From press to release.
    pub drag_ms: f64,
    /// Time the cursor was moving with the button held: the sum of the
    /// intervals between consecutive held movements, each no longer than
    /// [`MOVING_PAUSE_MS`].
    pub moving_ms: f64,
    pub coverage: Coverage,
    pub latencies: Vec<Latency>,
    /// Latencies timed from the consume, since no report of the extent
    /// preceded it.
    pub consume_starts: usize,
    pub unseen: usize,
    pub never_presented: Vec<[u32; 2]>,
    pub missing: Option<[u32; 2]>,
    pub gaps: Vec<Gap>,
    pub final_extent: [u32; 2],
    /// From release to the final extent's frame, if it was seen.
    pub final_ms: Option<f64>,
    /// Readings that name no submitted frame.
    pub unknown: usize,
    /// Frames seen that the host never accepted.
    pub unaccepted: usize,
    /// Changes of the visible frame to one older than a frame already seen.
    pub regressions: usize,
    /// Frames submitted during the drag at an extent other than the latest
    /// consumed one.
    pub stale_submissions: usize,
    /// Time during the drag the client showed a scaled frame, a frame drawn for
    /// another extent, or no readable stamp.
    pub stretched_ms: f64,
    pub mismatched_ms: f64,
    pub unreadable_ms: f64,
    /// Intervals between consecutive samples during the drag.
    pub sample_intervals: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sighting {
    pub frame: u64,
    pub extent: [u32; 2],
    pub submitted: i64,
    pub accepted: bool,
}

pub struct Clock {
    origin: i64,
    frequency: i64,
}

impl Clock {
    pub fn ms(&self, counter: i64) -> f64 {
        (counter - self.origin) as f64 * 1000.0 / self.frequency as f64
    }
}

/// The longest run of samples with the primary button held.
fn drag(samples: &[Sample]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut start = None;
    for (index, sample) in samples.iter().enumerate() {
        match (sample.pressed, start) {
            (true, None) => start = Some(index),
            (false, Some(first)) => {
                if best.is_none_or(|(a, b)| index - first > b - a) {
                    best = Some((first, index));
                }
                start = None;
            }
            _ => {}
        }
    }
    best
}

fn sighting(
    sample: &Sample,
    host: &[HostEvent],
    accepted: &BTreeSet<u64>,
) -> Option<Option<Sighting>> {
    let Reading::Frame(bits) = sample.reading else {
        return None;
    };
    let submitted = host.iter().rev().find_map(|event| match event.kind {
        HostKind::Submitted { frame, extent }
            if event.counter <= sample.after && frame & 0xffff == u64::from(bits) =>
        {
            Some((frame, extent, event.counter))
        }
        _ => None,
    });
    Some(submitted.map(|(frame, extent, counter)| Sighting {
        frame,
        extent,
        submitted: counter,
        accepted: accepted.contains(&frame),
    }))
}

/// The first report of `extent` in the window's latest run of reports of it at
/// or before `counter`.
fn pending_observation(events: &[HostEvent], counter: i64, extent: [u32; 2]) -> Option<i64> {
    let mut start = None;
    for event in events.iter().rev().filter(|event| event.counter <= counter) {
        match event.kind {
            HostKind::Observed(seen) if seen == extent => start = Some(event.counter),
            HostKind::Observed(_) if start.is_some() => break,
            _ => {}
        }
    }
    start
}

fn latest_consumed(events: &[HostEvent], counter: i64) -> Option<[u32; 2]> {
    events.iter().rev().find_map(|event| match event.kind {
        HostKind::Consumed(extent) if event.counter <= counter => Some(extent),
        _ => None,
    })
}

/// The longest stillness between two cursor movements that still counts as
/// moving, so a drag that pauses counts only its moving stretches.
pub const MOVING_PAUSE_MS: f64 = 250.0;

fn moving_ms(samples: &[Sample], [first, released]: [usize; 2], clock: &Clock) -> f64 {
    let moves: Vec<f64> = (first.max(1)..released)
        .filter(|&index| samples[index].cursor != samples[index - 1].cursor)
        .map(|index| clock.ms(samples[index].before))
        .collect();
    moves
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .filter(|&interval| interval <= MOVING_PAUSE_MS)
        .sum()
}

pub fn analyze(host: &HostTrace, capture: &CaptureLog) -> Result<Analysis, String> {
    if host.frequency != capture.frequency {
        return Err(format!(
            "the host counts {} per second but the capture {}; they do not share a clock",
            host.frequency, capture.frequency
        ));
    }
    let samples = &capture.samples;
    let (first, released) = drag(samples).ok_or("the capture holds no completed press")?;
    let press = samples[first].before;
    let release = samples[released - 1].before;
    let clock = Clock {
        origin: press,
        frequency: host.frequency,
    };
    let events = &host.events;
    let mut analysis = Analysis {
        press,
        drag_ms: clock.ms(release),
        moving_ms: moving_ms(samples, [first, released], &clock),
        ..Analysis::default()
    };

    let accepted: BTreeSet<u64> = events
        .iter()
        .filter_map(|event| match event.kind {
            HostKind::Accepted(frame) => Some(frame),
            _ => None,
        })
        .collect();
    let sightings: Vec<Option<Option<Sighting>>> = samples
        .iter()
        .map(|sample| sighting(sample, events, &accepted))
        .collect();
    let mut seen = BTreeSet::new();
    let mut shown: Option<u64> = None;
    for (index, sample) in samples.iter().enumerate() {
        if let Some(found) = sightings[index] {
            match found {
                None => analysis.unknown += 1,
                Some(frame) => {
                    if !frame.accepted && !seen.contains(&frame.frame) {
                        analysis.unaccepted += 1;
                    }
                    let newest = seen.last().copied();
                    if shown != Some(frame.frame)
                        && newest.is_some_and(|newest| frame.frame < newest)
                    {
                        analysis.regressions += 1;
                    }
                    seen.insert(frame.frame);
                    shown = Some(frame.frame);
                }
            }
        }
        if index <= first || index >= released {
            continue;
        }
        let interval = clock.ms(sample.after) - clock.ms(samples[index - 1].after);
        analysis.sample_intervals.push(interval);
        match (sample.reading, sightings[index]) {
            (Reading::Stretched(_), _) => analysis.stretched_ms += interval,
            (Reading::Unreadable, _) | (_, Some(None)) => analysis.unreadable_ms += interval,
            (_, Some(Some(frame))) if frame.extent != sample.client => {
                analysis.mismatched_ms += interval
            }
            _ => {}
        }
    }

    let consumed: Vec<(i64, [u32; 2])> = events
        .iter()
        .filter_map(|event| match event.kind {
            HostKind::Consumed(extent) => Some((event.counter, extent)),
            _ => None,
        })
        .collect();
    let during = |counter: i64| counter >= press && counter <= release;
    let extents: Vec<[u32; 2]> = consumed
        .iter()
        .filter(|(counter, _)| during(*counter))
        .map(|(_, extent)| *extent)
        .collect();
    analysis.coverage = coverage(&extents, capture.dpi);
    analysis.stale_submissions = events
        .iter()
        .filter(|event| match event.kind {
            HostKind::Submitted { extent, .. } if during(event.counter) => {
                latest_consumed(events, event.counter).is_some_and(|latest| latest != extent)
            }
            _ => false,
        })
        .count();

    for (position, &(counter, extent)) in consumed.iter().enumerate() {
        if !during(counter) {
            continue;
        }
        let next = consumed.get(position + 1).map(|(next, _)| *next);
        let within =
            |submitted: i64| submitted >= counter && next.is_none_or(|next| submitted < next);
        let visible = samples
            .iter()
            .zip(&sightings)
            .find_map(|(sample, found)| match found {
                Some(Some(frame))
                    if frame.accepted && frame.extent == extent && within(frame.submitted) =>
                {
                    Some(sample.after)
                }
                _ => None,
            });
        let presented = events.iter().any(|event| match event.kind {
            HostKind::Submitted {
                frame,
                extent: drawn,
            } => drawn == extent && within(event.counter) && accepted.contains(&frame),
            _ => false,
        });
        match (visible, next) {
            (Some(visible), _) => {
                let observed = pending_observation(events, counter, extent);
                analysis.consume_starts += usize::from(observed.is_none());
                analysis.latencies.push(Latency {
                    extent,
                    observed: clock.ms(observed.unwrap_or(counter)),
                    visible: clock.ms(visible),
                });
            }
            (None, None) => analysis.missing = Some(extent),
            (None, Some(_)) if presented => analysis.unseen += 1,
            (None, Some(_)) => analysis.never_presented.push(extent),
        }
    }

    analysis.gaps = gaps(samples, &sightings, events, [first, released], &clock);

    let last = samples.last().ok_or("the capture holds no samples")?;
    analysis.final_extent = last.client;
    let settled = events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, HostKind::Observed(_)))
        .map_or(release, |event| event.counter);
    analysis.final_ms = samples
        .iter()
        .zip(&sightings)
        .find_map(|(sample, found)| match found {
            Some(Some(frame))
                if sample.after >= settled
                    && frame.accepted
                    && frame.extent == last.client
                    && sample.client == last.client =>
            {
                Some((clock.ms(sample.after) - clock.ms(release)).max(0.0))
            }
            _ => None,
        });
    Ok(analysis)
}

#[cfg(test)]
#[path = "tests/analysis.rs"]
pub(crate) mod tests;
