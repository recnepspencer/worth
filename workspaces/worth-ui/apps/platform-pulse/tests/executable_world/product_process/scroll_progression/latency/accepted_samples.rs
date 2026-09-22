//! Ordered one-to-one correspondence, not a repeated-pose search over an input cycle.
use super::*;
use crate::product_process::PlatformPulseNativeCloseEvidence;
use std::collections::BTreeSet;

// Host completion follows GPU work/readback; compositor rendering can fall on
// either side. Two qualified60Hz intervals bound the join, less than one80ms
// input period. This is correspondence slack, never subtracted from latency.
const COMPOSITOR_JOIN_SLACK_100NS: u64 = 340_000;

#[derive(Clone, Copy, Debug)]
pub(super) struct AcceptedPose {
    pub(super) frame: u64,
    pub(super) attempt: u64,
    pub(super) qpc_100ns: i64,
    pub(super) top: i64,
    pub(super) length: i64,
}

#[derive(Debug)]
pub(super) struct JoinedSamples {
    pub(super) observed: Vec<(usize, usize)>,
    pub(super) unobserved: Vec<usize>,
}

pub(super) fn join(
    samples: &[AcceptedPose],
    captures: &[VisibleScrollFrame],
) -> Result<JoinedSamples, &'static str> {
    if samples.windows(2).any(|s| s[0].qpc_100ns >= s[1].qpc_100ns)
        || captures
            .windows(2)
            .any(|s| s[0].qpc_100ns >= s[1].qpc_100ns)
    {
        return Err("accepted/captured observations must be strictly ordered");
    }
    let mut identities = BTreeSet::new();
    let mut result = JoinedSamples {
        observed: Vec::new(),
        unobserved: Vec::new(),
    };
    let mut next_capture = 0;
    for (sample_index, sample) in samples.iter().enumerate() {
        if sample.qpc_100ns < 0 || !identities.insert((sample.frame, sample.attempt)) {
            return Err("accepted sample has invalid time or reused identity");
        }
        let matching = captures
            .iter()
            .enumerate()
            .skip(next_capture)
            .filter(|(_, frame)| {
                frame.qpc_100ns.abs_diff(sample.qpc_100ns) <= COMPOSITOR_JOIN_SLACK_100NS
                    && (i64::from(frame.thumb_top_px) - sample.top).abs() <= 2
                    && (i64::from(frame.thumb_length_px) - sample.length).abs() <= 3
            })
            .min_by_key(|(_, frame)| frame.qpc_100ns.abs_diff(sample.qpc_100ns));
        if let Some((capture_index, _)) = matching {
            result.observed.push((sample_index, capture_index));
            next_capture = capture_index + 1;
        } else {
            // A compositor may coalesce accepted presents, but missing evidence
            // is not proof that this specific pose was ever displayed.
            result.unobserved.push(sample_index);
        }
    }
    Ok(result)
}

impl ScrollLatencyEvidence {
    pub(crate) fn assert_accepted_samples(&self, accepted: &PlatformPulseNativeCloseEvidence) {
        let region = RecentActivityScrollGeometry.viewport_points();
        let mut samples = Vec::new();
        for sample in accepted.sample_frames() {
            for chrome in sample.sampled_chrome() {
                let bounds = chrome.bounds_milli();
                let center = (bounds[0] + bounds[2] / 2) as f64 / 1000.0;
                if chrome.inline()
                    || !chrome.thumb()
                    || (center - (region[0] + region[2] - 6.0)).abs() > 1.0
                {
                    continue;
                }
                let attempt = sample
                    .presentation_attempt()
                    .expect("sample names its exact attempt");
                assert_eq!(sample.presentation_epoch(), Some(attempt));
                samples.push(AcceptedPose {
                    frame: sample.frame(),
                    attempt,
                    qpc_100ns: sample
                        .accepted_qpc_100ns()
                        .expect("accepted sample has QPC completion time"),
                    top: physical_px(bounds[1] as f64 / 1000.0, self.dpi),
                    length: physical_px(bounds[3] as f64 / 1000.0, self.dpi),
                });
            }
        }
        let joined =
            join(&samples, &self.visible_frames).expect("valid ordered native observations");
        let observed = joined
            .observed
            .iter()
            .map(|&(s, c)| (samples[s], &self.visible_frames[c]))
            .collect::<Vec<_>>();
        let positions = observed
            .iter()
            .map(|(sample, _)| sample.top)
            .collect::<BTreeSet<_>>();
        println!("one-to-one accepted sample/pixel joins (34ms maximum correspondence slack): {observed:?}");
        println!(
            "accepted samples not separately observed (coalescence is possible, not proven): {:?}",
            joined
                .unobserved
                .iter()
                .map(|&i| samples[i])
                .collect::<Vec<_>>()
        );
        assert!(observed.len() >= 5 && positions.len() >= 3,
            "multiple distinct intermediate accepted positions must agree with external pixels: {observed:?}");
        println!(
            "host sample costs (damage, rendered pixels, submissions, presents): {:?}",
            accepted
                .sample_frames()
                .iter()
                .map(|s| (
                    s.logical_damage_regions(),
                    s.rendered_pixels(),
                    s.queue_submissions(),
                    s.presents()
                ))
                .collect::<Vec<_>>()
        );
    }
}
