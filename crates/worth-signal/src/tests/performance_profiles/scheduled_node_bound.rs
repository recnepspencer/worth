use std::time::Instant;

use crate::facade::SignalRuntimePolicy;
use crate::tests::domains::fintech::compile_financial_locality_world_with_policy;

use super::throughput_definition::{
    assert_within_throughput_budget, partitioned_world_for_output_floor, performance_workers,
    PERFORMANCE_SEED, RECORDED_SCHEDULED_OUTPUT_FLOOR,
};
const SCHEDULED_BATCHES: usize = 8;

#[test]
fn scheduled_node_bound_records_governing_scale() {
    let started = Instant::now();
    let mut world = compile_financial_locality_world_with_policy(
        partitioned_world_for_output_floor(PERFORMANCE_SEED, RECORDED_SCHEDULED_OUTPUT_FLOOR),
        SignalRuntimePolicy::operational(),
    )
    .expect("scheduled bound world compiles under installed operational policy");
    let report = world
        .run_locality_performance_sequence(SCHEDULED_BATCHES, performance_workers(), false)
        .expect("scheduled bound sequence settles");
    assert!(
        report.node_count >= RECORDED_SCHEDULED_OUTPUT_FLOOR as usize,
        "scheduled packet must exercise the recorded {RECORDED_SCHEDULED_OUTPUT_FLOOR}-node bound, got {}",
        report.node_count
    );
    println!(
        "scheduled node bound governed_by={RECORDED_SCHEDULED_OUTPUT_FLOOR} report={report:?}"
    );
    assert_within_throughput_budget(started, "scheduled recorded bound");
}
