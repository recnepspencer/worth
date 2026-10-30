//! Independent retained-history and live-body axes through ordinary cell edits.
#[path = "cost/measurement.rs"]
mod measurement;
#[path = "cost/peak.rs"]
mod peak;
#[path = "cost/reconstruction.rs"]
mod reconstruction;
#[path = "cost/workload.rs"]
mod workload;

use workload::CellHistory;

fn history_axis(scales: &[u64]) {
    report_runtime("retained_history", scales);
    // A fixed live body and one changed cell: publication may traverse Native
    // persistent paths, but must not copy or hash the entire retained history.
    let mut samples = Vec::new();
    for &history in scales {
        let court = CellHistory::build(3, history);
        samples.push(court.measure());
    }
    assert_sublinear_publication(&samples);
}

fn body_axis(scales: &[u64]) {
    report_runtime("live_body", scales);
    // The LAST cell is edited, so a first-match scan cannot counterfeit locality.
    // Historical materialization remains output-sized; its reads are reported
    // separately rather than charged to the warm publication interval.
    let mut samples = Vec::new();
    for &live in scales {
        let court = CellHistory::build(live, 1);
        samples.push(court.measure());
    }
    assert_sublinear_publication(&samples);
}

fn assert_sublinear_publication(samples: &[measurement::WarmPublication]) {
    for pair in samples.windows(2) {
        assert!(pair[1].requested_object_peak_bytes <= pair[0].requested_object_peak_bytes.saturating_mul(8),
            "local edit transient requested-object peak scales with the independent population: {pair:?}");
        // These axes grow by at least 100x. An 8x envelope permits persistent
        // path/physical-page growth, while rejecting body/history-sized copies.
        assert!(
            pair[1].copied_bytes <= pair[0].copied_bytes * 8,
            "local cell publication copied with the independent population: {pair:?}"
        );
        assert!(
            pair[1].hashed_values <= pair[0].hashed_values * 8,
            "local cell publication rehashed the independent population: {pair:?}"
        );
        assert!(
            pair[1].index_path_nodes <= pair[0].index_path_nodes * 8,
            "local cell publication rebuilt persistent indexes: {pair:?}"
        );
        assert!(
            pair[1].new_authoritative_bytes <= pair[0].new_authoritative_bytes * 8,
            "local cell publication allocated with the independent population: {pair:?}"
        );
        assert!(
            pair[1].decision_facts <= pair[0].decision_facts * 8,
            "local cell decisions scanned the independent population: {pair:?}"
        );
        assert!(
            pair[1].world_reachability_lookups <= pair[0].world_reachability_lookups * 8,
            "ordinary World retirement revisited the protected history: {pair:?}"
        );
        // This fixture has one physical root per version and no alias/fork fanout.
        // Ordinary work includes actual delta candidates plus the configured older
        // sweep. Below that granule, H=100 cannot distinguish a full scan from one
        // lawful bounded sweep; the required 10k case supplies that distinction.
        let granule = pair[0]
            .native_retention_sweep_granule
            .max(pair[1].native_retention_sweep_granule);
        assert!(pair[1].native_index_retention_roots_examined
            <= pair[0].native_index_retention_roots_examined.max(granule).saturating_mul(8),
            "ordinary Native index cleanup scanned retained history beyond its delta/sweep granule: {pair:?}");
    }
}

fn report_runtime(axis: &str, scales: &[u64]) {
    println!("cell workload=single-field-last-cell policy=Serial/CanonicalBitwise allocator=instrumented-tracking-system;wrapper-always-present;no-uninstrumented-latency-claim os={} arch={} available_logical_cpus={} axis={axis} scales={scales:?} expectation=affected-cell-plus-native-persistent-path; historical-output-sized; head-bytes-exclude-held-history", std::env::consts::OS, std::env::consts::ARCH, std::thread::available_parallelism().unwrap());
}

#[test]
fn cell_publication_and_history_reads_separate_the_two_small_scale_axes() {
    history_axis(&[1, 100]);
    body_axis(&[1, 100]);
}

#[test]
#[ignore = "scheduled qualification at ten thousand retained feature steps"]
fn cell_publication_and_history_reads_qualify_ten_thousand_retained_steps() {
    history_axis(&[1, 100, 10_000]);
}

#[test]
#[ignore = "scheduled qualification at ten thousand live cells"]
fn cell_publication_and_history_reads_qualify_ten_thousand_live_cells() {
    body_axis(&[1, 100, 10_000]);
}
