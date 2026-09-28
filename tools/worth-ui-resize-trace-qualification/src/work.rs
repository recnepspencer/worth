//! The host's text, target, and swapchain work during the drag, the span of
//! each stage of its frames, and the most of each resource it retained.
//! Reported beside the timings, not graded: the milestone asks for them in the
//! evidence but sets no limit on them.

use crate::logs::{HostKind, HostTrace};

/// The counts a `text` trace line lists after its frame, in order.
pub const TEXT_WORK: [&str; 5] = [
    "shaped runs",
    "shaped scalars",
    "positioned glyphs",
    "emitted lines",
    "rasterized glyphs",
];

/// The stages a `stage` trace line names, as the host writes them.
pub const STAGES: [&str; 13] = [
    "frame",
    "settle",
    "projection",
    "appearance",
    "async",
    "layout",
    "completion",
    "atlas",
    "draw",
    "acquire",
    "encode",
    "submit",
    "surface",
];

#[derive(Debug, Default, PartialEq)]
pub struct Work {
    /// Extents of the render targets allocated during the drag.
    pub targets: Vec<[u32; 2]>,
    /// Extents the swapchain was configured at during the drag.
    pub swapchains: Vec<[u32; 2]>,
    /// Presentation attempts during the drag whose text work the host traced.
    pub text_attempts: usize,
    /// Of those, the attempts that shaped any run.
    pub shaping_attempts: usize,
    /// Each [`TEXT_WORK`] count summed over the drag.
    pub text_total: [u64; 5],
    /// Each [`TEXT_WORK`] count's largest value in one attempt.
    pub text_max: [u64; 5],
    /// The milliseconds each span of each [`STAGES`] stage took, for the
    /// spans that ended during the drag.
    pub stage_ms: [Vec<f64>; STAGES.len()],
    /// The most of each resource the host retained at once, over the whole
    /// run rather than the drag.
    pub peaks: Vec<(String, u64)>,
}

/// The work the host traced between `press` and `release`.
pub fn during(host: &HostTrace, [press, release]: [i64; 2]) -> Work {
    let mut work = Work {
        peaks: host.peaks.clone(),
        ..Work::default()
    };
    let drag = host
        .events
        .iter()
        .filter(|event| event.counter >= press && event.counter <= release);
    for event in drag {
        match event.kind {
            HostKind::Target(extent) => work.targets.push(extent),
            HostKind::Swapchain(extent) => work.swapchains.push(extent),
            HostKind::Text { work: counts, .. } => {
                work.text_attempts += 1;
                work.shaping_attempts += usize::from(counts[0] > 0);
                for (index, count) in counts.into_iter().enumerate() {
                    work.text_total[index] += count;
                    work.text_max[index] = work.text_max[index].max(count);
                }
            }
            HostKind::Stage { stage, start } => {
                let counts = (event.counter - start) as f64;
                work.stage_ms[stage].push(counts * 1000.0 / host.frequency as f64);
            }
            _ => {}
        }
    }
    work
}

#[cfg(test)]
#[path = "tests/work.rs"]
mod tests;
