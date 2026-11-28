use super::observation::{Metric, Sample};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Axis {
    Model,
    History,
    Installed,
}

#[derive(Debug)]
pub struct GrowthFailure {
    pub metric: Metric,
    pub axis: Axis,
    pub baseline: u64,
    pub doubled: u64,
}

impl std::fmt::Display for GrowthFailure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ratio = if self.baseline == 0 {
            f64::INFINITY
        } else {
            self.doubled as f64 / self.baseline as f64
        };
        write!(
            out,
            "{:?}/{} axis={:?}: {} -> {}, ratio={ratio:.3} (requires <1.5)",
            self.metric.phase, self.metric.name, self.axis, self.baseline, self.doubled
        )
    }
}

pub fn compare(samples: &[Sample; 4]) -> Vec<GrowthFailure> {
    let baseline = &samples[0];
    assert_eq!(
        samples.each_ref().map(|s| s.dimensions),
        baseline.dimensions.points(),
        "scale points must vary exactly one independent axis"
    );
    let mut failures = Vec::new();
    for (sample, axis) in samples[1..]
        .iter()
        .zip([Axis::Model, Axis::History, Axis::Installed])
    {
        assert_eq!(
            sample.commits, baseline.commits,
            "commits per edit changed on {axis:?}"
        );
        assert_eq!(
            sample.touched, baseline.touched,
            "touched records changed on {axis:?}"
        );
        assert!(
            sample.work.keys().eq(baseline.work.keys()),
            "observation coverage changed on {axis:?}"
        );
        for (&metric, &before) in &baseline.work {
            let after = sample.work[&metric];
            // Zero work stays zero. Integer cross multiplication preserves the
            // strict bound without rounding or a fixture-fitted additive floor.
            if after > 0 && u128::from(after) * 2 >= u128::from(before) * 3 {
                failures.push(GrowthFailure {
                    metric,
                    axis,
                    baseline: before,
                    doubled: after,
                });
            }
        }
    }
    failures
}
