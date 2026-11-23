use super::*;

#[derive(Debug, Clone)]
pub(super) struct CountedPerfSample {
    pub(super) charged_work: u64,
    pub(super) metrics: PerfMetricSet,
}

pub(super) fn capture_counted_samples(
    mut run: impl FnMut() -> CountedPerfSample,
) -> Vec<CountedPerfSample> {
    let mut samples = Vec::with_capacity(perf_samples());
    for _ in 0..perf_samples() {
        samples.push(run());
    }
    samples
}

pub(super) fn assert_counted_budget(
    samples: &[CountedPerfSample],
    description: &str,
    predicate: impl Fn(&PerfMetricSet) -> bool,
) {
    if let Some((index, sample)) = samples
        .iter()
        .enumerate()
        .find(|(_, sample)| !predicate(&sample.metrics))
    {
        panic!(
            "performance budget failed: {description}\nsample[{index}] metrics: {:?}",
            sample.metrics
        );
    }
}

pub(super) fn commit_measurement(
    runtime: &RelationalRuntime,
    run: impl FnOnce(&RelationalRuntime) -> (CommitResult, u64),
) -> CountedPerfSample {
    runtime.performance_access().reset_counters();
    let (outcome, charged_work) = run(runtime);
    let counters = runtime.performance_access().counters();

    CountedPerfSample {
        charged_work,
        metrics: perf_metrics!({
            "charged_work": charged_work,
            "changed_records": outcome.changed_records.len(),
            "commit_topology": format!("{:?}", outcome.structural_summary().commit_topology),
            "touched_partitions": outcome.structural_summary().touched_partitions.len(),
            "packet_count": outcome.complexity_delta().preparation_packet_count,
            "query_packet_count": outcome.complexity_delta().query_packet_count,
            "snapshot_pin_full_rebuilds": outcome.complexity_delta().snapshot_pin_full_rebuilds,
            "durable_log_len": runtime.durable_log().len(),
            "counters": counters,
        }),
    }
}
