use serde::{Deserialize, Serialize};

pub(super) const MATCHED_PROTOCOL_ID: &str = "signal-complete-family-abba-v2";
pub(super) const MATCHED_CAPTURE_ORDER: [&str; 4] = ["A1", "B1", "B2", "A2"];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PerfTimingPolicy {
    StrictHeavy,
    MedianOnly,
    StructuralOnly,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub(super) struct PerfCaseResolution {
    sample_count: usize,
    warmup_count: usize,
}

impl PerfCaseResolution {
    pub(super) const fn sample_count(self) -> usize {
        self.sample_count
    }

    pub(super) const fn warmup_count(self) -> usize {
        self.warmup_count
    }
}

pub(super) fn case_resolution(policy: PerfTimingPolicy) -> PerfCaseResolution {
    reject_sample_overrides();
    match policy {
        PerfTimingPolicy::StrictHeavy | PerfTimingPolicy::MedianOnly => PerfCaseResolution {
            sample_count: 21,
            warmup_count: 4,
        },
        PerfTimingPolicy::StructuralOnly => PerfCaseResolution {
            sample_count: 3,
            warmup_count: 0,
        },
    }
}

fn reject_sample_overrides() {
    for name in ["WORTH_SIGNAL_PERF_SAMPLES", "WORTH_SIGNAL_PERF_WARMUPS"] {
        assert!(
            std::env::var_os(name).is_none(),
            "{name} cannot override the frozen matched measurement protocol"
        );
    }
}

pub(super) fn case_protocol(policy: PerfTimingPolicy) -> serde_json::Value {
    let resolution = case_resolution(policy);
    serde_json::json!({
        "id": MATCHED_PROTOCOL_ID,
        "capture_order": MATCHED_CAPTURE_ORDER,
        "sample_count": resolution.sample_count(),
        "warmup_count": resolution.warmup_count(),
        "repeatability_rule": "symmetric A/A noise must fit conservative paired budget headroom",
        "statistical_posture": "order statistics; not a confidence interval",
    })
}

#[cfg(test)]
mod tests {
    use super::{case_resolution, PerfTimingPolicy, MATCHED_CAPTURE_ORDER};

    #[test]
    fn f2_resolution_and_counterbalanced_order_are_frozen() {
        for policy in [PerfTimingPolicy::StrictHeavy, PerfTimingPolicy::MedianOnly] {
            let resolution = case_resolution(policy);
            assert_eq!(resolution.sample_count(), 21);
            assert_eq!(resolution.warmup_count(), 4);
        }
        let structural = case_resolution(PerfTimingPolicy::StructuralOnly);
        assert_eq!(structural.sample_count(), 3);
        assert_eq!(structural.warmup_count(), 0);
        assert_eq!(MATCHED_CAPTURE_ORDER, ["A1", "B1", "B2", "A2"]);
    }
}
