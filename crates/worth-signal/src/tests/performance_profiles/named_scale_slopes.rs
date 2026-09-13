use std::time::Instant;

use crate::facade::{SignalRuntimePolicy, StageExecutor};
use crate::tests::domains::fintech::{
    compile_financial_locality_world_with_policy, DensityRatio, FinancialWorldDefinition,
};

use super::throughput_definition::{
    assert_within_throughput_budget, ordinary_definition, partitioned_world_for_output_floor,
    performance_executor, PERFORMANCE_SEED,
};

#[derive(Debug)]
struct ScaleSlopeEvidence {
    axis: &'static str,
    lower_scale: usize,
    lower_micros: u128,
    upper_scale: usize,
    upper_micros: u128,
}

#[test]
fn named_scale_slopes_record_independent_axes() {
    let started = Instant::now();
    let (evidence, parallel_dispatch_sample) = run_scale_slopes();
    let nodes = evidence
        .iter()
        .find(|row| row.axis == "nodes")
        .expect("nodes axis");
    assert!(
        nodes.upper_scale > nodes.lower_scale,
        "node slope must change only the node axis: {nodes:?}"
    );
    assert!(nodes.lower_micros > 0 && nodes.upper_micros > 0);
    let fanout = evidence
        .iter()
        .find(|row| row.axis == "fanout")
        .expect("fanout axis");
    assert!(
        fanout.upper_scale > fanout.lower_scale,
        "fanout slope must grow downstream instruments at fixed region count: {fanout:?}"
    );
    assert!(fanout.lower_micros > 0 && fanout.upper_micros > 0);
    let edit_width = evidence
        .iter()
        .find(|row| row.axis == "edit_width")
        .expect("edit width axis");
    assert_eq!(edit_width.lower_scale, 1);
    assert_eq!(edit_width.upper_scale, 4);
    assert!(edit_width.lower_micros > 0 && edit_width.upper_micros > 0);
    println!("named scale slopes={evidence:?} parallel_dispatch_sample={parallel_dispatch_sample}");
    assert_within_throughput_budget(started, "named scale slopes");
}

#[test]
fn disjoint_region_batch_program_records_scope_mix() {
    let started = Instant::now();
    const DISJOINT_BATCHES: usize = 8;
    let mut world = compile_financial_locality_world_with_policy(
        ordinary_definition(),
        SignalRuntimePolicy::operational(),
    )
    .expect("disjoint-region world compiles");
    let report = world
        .run_locality_performance_sequence(DISJOINT_BATCHES, performance_executor(), false)
        .expect("disjoint-region sequence settles");
    assert!(report.node_count >= 1_024);
    assert_eq!(report.batch_count, DISJOINT_BATCHES);
    assert!(report.peak_touched_nodes > 0);
    assert!(report.semantic_work_rows.iter().all(|row| !row.is_empty()));
    assert_within_throughput_budget(started, "disjoint-region batch program");
}

fn run_scale_slopes() -> (Vec<ScaleSlopeEvidence>, u128) {
    let node_lower = slope_partitioned_median(256);
    let node_upper = slope_partitioned_median(1_024);
    let fanout_lower = slope_world_median(FinancialWorldDefinition::partitioned_curve_universe(
        PERFORMANCE_SEED,
        256,
        1,
        1,
    ));
    let fanout_upper = slope_world_median(FinancialWorldDefinition::partitioned_curve_universe(
        PERFORMANCE_SEED,
        256,
        1,
        4,
    ));
    let edit_width = [DensityRatio::OneInOneHundred, DensityRatio::OneInFour]
        .into_iter()
        .map(|affected_ratio| {
            slope_world_median(FinancialWorldDefinition::dense_market_close(
                PERFORMANCE_SEED,
                1_024,
                affected_ratio,
            ))
        })
        .collect::<Vec<_>>();
    let evidence = vec![
        ScaleSlopeEvidence {
            axis: "nodes",
            lower_scale: node_lower.0,
            lower_micros: node_lower.1,
            upper_scale: node_upper.0,
            upper_micros: node_upper.1,
        },
        ScaleSlopeEvidence {
            axis: "edit_width",
            lower_scale: 1,
            lower_micros: edit_width[0].1,
            upper_scale: 4,
            upper_micros: edit_width[1].1,
        },
        ScaleSlopeEvidence {
            axis: "fanout",
            lower_scale: fanout_lower.0,
            lower_micros: fanout_lower.1,
            upper_scale: fanout_upper.0,
            upper_micros: fanout_upper.1,
        },
    ];
    #[cfg(feature = "parallel")]
    let parallel_dispatch_sample = {
        let lower = parallel_dispatch_sample(256);
        let upper = parallel_dispatch_sample(1_024);
        assert!(lower.1 > 0 && upper.1 > 0);
        assert!(upper.0 > lower.0);
        upper.1
    };
    #[cfg(not(feature = "parallel"))]
    let parallel_dispatch_sample = 0;
    (evidence, parallel_dispatch_sample)
}

fn slope_partitioned_median(output_floor: u32) -> (usize, u128) {
    let report_median = slope_world_median(partitioned_world_for_output_floor(
        PERFORMANCE_SEED,
        output_floor,
    ));
    (report_median.0, report_median.1)
}

fn slope_world_median(definition: FinancialWorldDefinition) -> (usize, u128) {
    let mut world = compile_financial_locality_world_with_policy(
        definition,
        SignalRuntimePolicy::operational(),
    )
    .expect("slope world compiles under installed operational policy");
    let report = world
        .run_locality_performance_sequence(8, StageExecutor::Serial, false)
        .expect("slope sequence settles");
    (report.node_count, report.warm_median_micros)
}

#[cfg(feature = "parallel")]
fn parallel_dispatch_sample(output_floor: u32) -> (usize, u128) {
    let mut world = compile_financial_locality_world_with_policy(
        FinancialWorldDefinition::dense_market_close(
            PERFORMANCE_SEED,
            output_floor,
            DensityRatio::FourInFive,
        ),
        SignalRuntimePolicy::operational(),
    )
    .expect("parallel-dispatch slope world compiles");
    let report = world
        .run_locality_performance_sequence(8, StageExecutor::balanced_parallel(), false)
        .expect("parallel-dispatch slope settles");
    assert!(report.parallel_stage_dispatches > 0);
    (report.node_count, report.warm_median_micros)
}
