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
fn measured_idle_versus_introspective_records_diagnostic_pressure_benefit() {
    let started = Instant::now();
    let (idle_report, pressure_report, idle_inventory, pressure_inventory) = pressure_reports();
    assert_profile_report(
        &idle_report,
        ORDINARY_OUTPUT_FLOOR as usize,
        PERFORMANCE_BATCHES,
    );
    assert_profile_report(
        &pressure_report,
        ORDINARY_OUTPUT_FLOOR as usize,
        PERFORMANCE_BATCHES,
    );
    assert!(idle_inventory.is_idle_zero());
    assert!(!pressure_inventory.is_idle_zero());
    let ten_percent = ten_percent_benefit(&idle_report, &pressure_report);
    println!(
        "pressure packet context={} idle_median={} idle_p95={} idle_milli={} pressure_median={} pressure_p95={} pressure_milli={} ten_percent_benefit={ten_percent} governed_nodes={}",
        PerformancePacketContext::recorded(),
        idle_report.warm_median_micros,
        idle_report.warm_p95_micros,
        idle_report.completed_batches_per_second_milli,
        pressure_report.warm_median_micros,
        pressure_report.warm_p95_micros,
        pressure_report.completed_batches_per_second_milli,
        ORDINARY_OUTPUT_FLOOR
    );
    assert_within_throughput_budget(started, "measured idle/introspective pressure packet");
}

#[test]
#[ignore = "timed comparison; run alone on a quiet machine with -- --ignored --nocapture"]
fn measured_idle_versus_introspective_compares_wall_clock_throughput() {
    let (idle_report, pressure_report, _, _) = pressure_reports();
    let ten_percent = ten_percent_benefit(&idle_report, &pressure_report);
    assert!(
        ten_percent,
        "idle must improve completed-work throughput by at least 10% against introspective diagnostic pressure"
    );
}

fn ten_percent_benefit(
    idle_report: &FinancialPerformanceBatchReport,
    pressure_report: &FinancialPerformanceBatchReport,
) -> bool {
    idle_report.completed_batches_per_second_milli
        >= pressure_report.completed_batches_per_second_milli * 11 / 10
}

type ProfileReports = (
    FinancialPerformanceBatchReport,
    FinancialPerformanceBatchReport,
    LocalityOptionalObservationInventory,
    LocalityOptionalObservationInventory,
);

fn pressure_reports() -> ProfileReports {
    let idle = profiles()
        .into_iter()
        .find(|profile| profile.name == "throughput_idle")
        .expect("idle profile");
    let introspective = profiles()
        .into_iter()
        .find(|profile| profile.name == "introspective")
        .expect("introspective profile");
    let definition = ordinary_definition();
    let (idle_report, _, idle_inventory) =
        operational_digest_for(idle, definition.clone(), PERFORMANCE_BATCHES);
    let (pressure_report, _, pressure_inventory) =
        operational_digest_for(introspective, definition, PERFORMANCE_BATCHES);
    (
        idle_report,
        pressure_report,
        idle_inventory,
        pressure_inventory,
    )
}
