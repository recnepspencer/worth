mod actions;
mod certification;
mod comparisons;
mod complexity;
mod failure_injection;
mod fixture;
mod invariants;
mod naming;
mod probes;
mod scales;
mod scenarios;
mod workflows;

use crate::facade::history::BranchId;
use crate::facade::transactions::CommitResult;

pub(crate) use fixture::FintechWorld;
pub(crate) use probes::{CaseTruthProbe, ObservabilityProbe};

pub(crate) fn setup_intraday_risk_perf_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    scenarios::setup_world_for(execution, scenarios::FintechScenario::IntradayRisk)
}

pub(crate) fn setup_trade_correction_perf_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> FintechWorld {
    scenarios::setup_world_for(execution, scenarios::FintechScenario::LateTradeCorrection)
}

pub(crate) fn perf_open_analysis_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
) -> BranchId {
    actions::open_analysis_branch(execution, world)
}

pub(crate) fn perf_stress_intraday_risk(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    actions::stress_seeded_intraday_risk(execution, world, branch_id)
}

pub(crate) fn perf_correct_trade_correction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    actions::correct_seeded_trade_candidate(execution, world, branch_id)
}

pub(crate) fn perf_emit_trade_correction_audit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut FintechWorld,
    branch_id: BranchId,
) -> CommitResult {
    actions::emit_trade_correction_audit_record(execution, world, branch_id)
}

pub(crate) fn perf_capture_baseline_observability(world: &FintechWorld) -> ObservabilityProbe {
    probes::capture_observability_probe(world, probes::ProbeStage::Baseline)
}

pub(crate) fn perf_capture_post_mutation_observability(world: &FintechWorld) -> ObservabilityProbe {
    probes::capture_observability_probe(world, probes::ProbeStage::PostMutation)
}

pub(crate) fn perf_capture_intraday_risk_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
) -> CaseTruthProbe {
    probes::capture_case_truth_probe(
        execution,
        world,
        fixture::FintechCaseRole::IntradayRisk,
        probes::ProbeStage::PostMutation,
    )
}

pub(crate) fn perf_capture_trade_correction_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &FintechWorld,
) -> CaseTruthProbe {
    probes::capture_case_truth_probe(
        execution,
        world,
        fixture::FintechCaseRole::LateTradeCorrection,
        probes::ProbeStage::PostMutation,
    )
}
