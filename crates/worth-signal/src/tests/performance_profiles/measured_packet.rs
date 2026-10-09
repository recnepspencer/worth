use crate::tests::domains::fintech::{
    FinancialPerformanceBatchReport, LocalityOptionalObservationInventory,
};

use std::time::Instant;

use super::throughput_definition::{
    assert_profile_report, assert_within_throughput_budget, operational_digest_for,
    ordinary_definition, profiles, PerformancePacketContext, ORDINARY_OUTPUT_FLOOR,
    PERFORMANCE_BATCHES,
};

#[test]
fn measured_idle_versus_balanced_records_named_benefit() {
    let started = Instant::now();
    let (balanced_report, idle_report, balanced_inventory, idle_inventory) = packet_reports();
    assert_profile_report(
        &balanced_report,
        ORDINARY_OUTPUT_FLOOR as usize,
        PERFORMANCE_BATCHES,
    );
    assert_profile_report(
        &idle_report,
        ORDINARY_OUTPUT_FLOOR as usize,
        PERFORMANCE_BATCHES,
    );
    assert!(idle_inventory.is_idle_zero());
    assert!(!balanced_inventory.is_idle_zero());
    let ten_percent = idle_report.completed_batches_per_second_milli
        >= balanced_report.completed_batches_per_second_milli * 11 / 10;
    println!(
        "measured packet context={} idle={:?} balanced={:?} ten_percent_benefit={ten_percent} governed_nodes={}",
        PerformancePacketContext::recorded(),
        idle_report,
        balanced_report,
        ORDINARY_OUTPUT_FLOOR
    );
    assert_within_throughput_budget(started, "measured idle/balanced packet");
}

#[test]
#[ignore = "timed comparison; run alone on a quiet machine with -- --ignored --nocapture"]
fn measured_idle_versus_balanced_compares_wall_clock_reports() {
    let (balanced_report, idle_report, _, _) = packet_reports();
    assert!(idle_report.warm_median_micros * 100 <= balanced_report.warm_median_micros * 103);
    assert!(idle_report.warm_p95_micros * 100 <= balanced_report.warm_p95_micros * 105);
}

type ProfileReports = (
    FinancialPerformanceBatchReport,
    FinancialPerformanceBatchReport,
    LocalityOptionalObservationInventory,
    LocalityOptionalObservationInventory,
);

fn packet_reports() -> ProfileReports {
    let balanced = profiles()
        .into_iter()
        .find(|profile| profile.name == "balanced_continuous")
        .expect("balanced profile");
    let idle = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile");
    let definition = ordinary_definition();
    let (balanced_report, _, balanced_inventory) =
        operational_digest_for(balanced, definition.clone(), PERFORMANCE_BATCHES);
    let (idle_report, _, idle_inventory) =
        operational_digest_for(idle, definition, PERFORMANCE_BATCHES);
    (
        balanced_report,
        idle_report,
        balanced_inventory,
        idle_inventory,
    )
}
