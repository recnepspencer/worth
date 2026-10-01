#[path = "signal_parallel_perf_report/workloads.rs"]
mod workloads;

mod leased_signal_support;

use serde::Serialize;
use worth_signal::facade::runtime::RuntimePolicy;
use worth_signal::facade::specialist::ExecutionReport;
use worth_signal::facade::{Aspect, AspectVersion};

const ASPECT_A: Aspect = Aspect::new(0);

fn version_ab(a: u64, b: u64) -> AspectVersion {
    AspectVersion::from_updates([(ASPECT_A, a), (Aspect::new(1), b)])
}

#[derive(Debug, Serialize)]
struct PerfRecord {
    workload: &'static str,
    executor_profile: &'static str,
    runtime_policy: &'static str,
    core_storage_profile: &'static str,
    stage_parallel_admission_reasons: Vec<&'static str>,
    plan_task_count: u32,
    plan_stage_count: u32,
    planning_nanos: u64,
    execute_elapsed_nanos: u64,
    snapshot_nanos: u64,
    precompute_nanos: u64,
    apply_nanos: u64,
    semantic_finalize_nanos: u64,
    residual_nanos: u64,
    semantic_segment_count: u32,
    tasks_executed: u32,
    execution_reports: Vec<worth_foundational::ExecutionReport>,
}

fn nanos(value: u128) -> u64 {
    value.min(u64::MAX as u128) as u64
}

fn worker_profiles() -> [(&'static str, usize); 3] {
    [("workers-1", 1), ("workers-2", 2), ("workers-4", 4)]
}

fn summarize(
    workload: &'static str,
    executor_profile: &'static str,
    runtime_policy: &'static str,
    planning_nanos: u128,
    execute_elapsed_nanos: u128,
    report: &ExecutionReport,
) -> PerfRecord {
    let phase_total = report.execution_snapshot_nanos
        + report.stage_precompute_nanos
        + report.stage_apply_nanos
        + report.semantic_finalize_nanos;
    PerfRecord {
        workload,
        executor_profile,
        runtime_policy,
        core_storage_profile: "public-facade-default",
        stage_parallel_admission_reasons: report
            .stages
            .iter()
            .filter_map(|stage| stage.parallel_admission_reason.map(|reason| reason.code()))
            .collect(),
        plan_task_count: report.plan_summary.task_count,
        plan_stage_count: report.plan_summary.stage_count,
        planning_nanos: nanos(planning_nanos),
        execute_elapsed_nanos: nanos(execute_elapsed_nanos),
        snapshot_nanos: nanos(report.execution_snapshot_nanos),
        precompute_nanos: nanos(report.stage_precompute_nanos),
        apply_nanos: nanos(report.stage_apply_nanos),
        semantic_finalize_nanos: nanos(report.semantic_finalize_nanos),
        residual_nanos: nanos(execute_elapsed_nanos.saturating_sub(phase_total)),
        semantic_segment_count: report.semantic_segment_count,
        tasks_executed: report.tasks_executed,
        execution_reports: report.execution.clone(),
    }
}

fn runtime_policy_profiles() -> [(&'static str, RuntimePolicy); 3] {
    [
        ("operational", RuntimePolicy::operational()),
        ("development", RuntimePolicy::development()),
        ("forensic", RuntimePolicy::forensic()),
    ]
}

fn main() {
    let host = leased_signal_support::host();
    let mut records = Vec::new();
    for (runtime_policy_name, runtime_policy) in runtime_policy_profiles() {
        for (executor_profile, workers) in worker_profiles() {
            records.push(workloads::run_deep_chain(
                executor_profile,
                runtime_policy_name,
                runtime_policy,
                workers,
                &host,
            ));
            records.push(workloads::run_wide_stage(
                executor_profile,
                runtime_policy_name,
                runtime_policy,
                workers,
                &host,
            ));
            records.push(workloads::run_partition_tolerance(
                executor_profile,
                runtime_policy_name,
                runtime_policy,
                workers,
                &host,
            ));
        }
    }
    println!("{}", serde_json::to_string_pretty(&records).unwrap());
}
