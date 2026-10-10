//! Harness capability adaptation for the Signal bridge.

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_harness::facade::{
    AdapterSupport, CaptureDepth, ClockDomain, ComparisonMode, DiagnosticsLevel, ExecutionMode,
    ExecutionPhase, ExecutionProfile, ExecutionRequest, HarnessAdapter, HarnessCapabilities,
    RunRecord, ScenarioFixture, SnapshotRecord,
};

use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationRequestMode;
use worth_foundational::{ExecutionPosture, ExecutionRequestPolicy};

use super::super::runtime::{SignalFixtureFactory, SignalHarnessSession, SignalMutationAction};
use super::projection::RunProjection;
use super::request::{build_dirty_batch, resolve_targets};
use super::SignalHarnessBridge;

impl HarnessAdapter for SignalHarnessBridge {
    type Runtime = SignalHarnessSession;
    type Fixture = SignalFixtureFactory;
    type Mutation = SignalMutationAction;
    type TargetId = String;
    type Error = SignalError;

    fn adapter_name(&self) -> &'static str {
        "worth-signal"
    }

    fn capabilities(&self) -> HarnessCapabilities {
        let mut execution_modes = BTreeSet::new();
        execution_modes.insert(ExecutionMode::RuntimeDefault);
        execution_modes.insert(ExecutionMode::Serial);
        execution_modes.insert(ExecutionMode::StagedParallel);
        execution_modes.insert(ExecutionMode::FullParallel);

        let mut diagnostics_levels = BTreeSet::new();
        diagnostics_levels.insert(DiagnosticsLevel::Off);
        diagnostics_levels.insert(DiagnosticsLevel::Operational);
        diagnostics_levels.insert(DiagnosticsLevel::Development);
        diagnostics_levels.insert(DiagnosticsLevel::Forensic);

        let mut capture_depths = BTreeSet::new();
        capture_depths.insert(CaptureDepth::Minimal);
        capture_depths.insert(CaptureDepth::Standard);
        capture_depths.insert(CaptureDepth::Rich);

        let mut comparison_modes = BTreeSet::new();
        comparison_modes.insert(ComparisonMode::Exact);
        comparison_modes.insert(ComparisonMode::Semantic);
        let mut clock_domains = BTreeSet::new();
        clock_domains.insert(ClockDomain::Logical);
        let mut execution_phases = BTreeSet::new();
        execution_phases.insert(ExecutionPhase::Evaluate);

        let mut rich_record_kinds = BTreeSet::new();
        rich_record_kinds.insert("execution_report".to_string());
        rich_record_kinds.insert("graph_diagnostics".to_string());
        rich_record_kinds.insert("node_explanation".to_string());
        rich_record_kinds.insert("graph_metrics".to_string());

        HarnessCapabilities {
            execution_modes,
            diagnostics_levels,
            capture_depths,
            comparison_modes,
            clock_domains,
            execution_phases,
            replay_support: AdapterSupport::Supported,
            lineage_support: AdapterSupport::Supported,
            provenance_support: AdapterSupport::Supported,
            event_stream_support: AdapterSupport::Unsupported,
            performance_counter_support: AdapterSupport::Supported,
            workload_budget_support: AdapterSupport::Unsupported,
            attachment_support: AdapterSupport::Supported,
            rich_record_kinds,
        }
    }

    fn create_runtime(&self) -> Result<Self::Runtime, Self::Error> {
        Ok(SignalHarnessSession { runtime: None })
    }

    fn load_fixture(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
    ) -> Result<(), Self::Error> {
        runtime.runtime = Some(fixture.fixture.build_runtime()?);
        Ok(())
    }

    fn apply_mutation_batch(
        &self,
        runtime: &mut Self::Runtime,
        batch: &worth_harness::facade::MutationBatch<Self::Mutation>,
    ) -> Result<(), Self::Error> {
        let runtime = runtime.runtime_mut()?;
        let dirty = build_dirty_batch(runtime, batch)?;
        crate::logic::invalidation::mark_dirty_batch(runtime.graph_mut(), &dirty)?;
        Ok(())
    }

    fn execute(
        &self,
        runtime: &mut Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<RunRecord<Self::TargetId>, Self::Error> {
        let runtime = runtime.runtime_mut()?;
        let runtime_policy = Self::runtime_policy(profile.diagnostics_level);
        let targets = resolve_targets(runtime, &request.targets)?;
        runtime.graph.set_runtime_policy(runtime_policy);

        let evaluator = Arc::clone(&runtime.evaluator);
        let report = match Self::posture(profile.execution_mode) {
            ExecutionPosture::Serial => {
                let plan = runtime
                    .graph
                    .build_evaluation_plan(&targets, EvaluationRequestMode::Default)?;
                #[cfg(test)]
                if Self::requires_condition_aware_execution(&runtime.graph, &plan)? {
                    let mut comparator = crate::data::comparator::DefaultComparatorPolicyResolver {
                        fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
                        custom: crate::data::comparator::DefaultComparatorResolver,
                    };
                    let mut condition = crate::logic::evaluation::DefaultConditionResolver;
                    crate::logic::planner::execute_test_prepared_plan_with_resolvers(
                        &mut runtime.graph,
                        &plan,
                        &(),
                        &move |ctx| evaluator.evaluate(ctx),
                        &mut comparator,
                        &mut condition,
                    )?
                } else {
                    runtime
                        .graph
                        .execute_prepared_plan(&plan, &(), &move |ctx| evaluator.evaluate(ctx))?
                }
                #[cfg(not(test))]
                runtime
                    .graph
                    .execute_prepared_plan(&plan, &(), &move |ctx| evaluator.evaluate(ctx))?
            }
            ExecutionPosture::Automatic => {
                let checked = runtime.checked_evaluator.as_ref().ok_or_else(|| {
                    SignalError::invalid_input(
                        "parallel Signal harness fixture requires a checked evaluator",
                    )
                })?;
                let authority = runtime.execution_authority.as_ref().ok_or_else(|| {
                    SignalError::invalid_input(
                        "parallel Signal harness fixture requires an injected execution authority",
                    )
                })?;
                let mut lease_request = runtime.lease_request.clone().ok_or_else(|| {
                    SignalError::invalid_input(
                        "parallel Signal harness fixture requires a lease request",
                    )
                })?;
                lease_request.policy = ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    lease_request.policy.determinism(),
                    lease_request.policy.budget(),
                );
                let lease = authority
                    .request_lease(lease_request)
                    .map_err(SignalError::execution_admission_denied)?;
                runtime.graph.evaluate_checked(
                    &targets,
                    EvaluationRequestMode::Default,
                    &(),
                    &|ctx| checked.evaluate_checked(ctx),
                    worth_execution::ExecutionRequest::leased(&lease),
                )?
            }
        };

        let target_statuses = SignalHarnessBridge::target_statuses(runtime, &request.targets)?;
        Ok(SignalHarnessBridge::run_record(RunProjection {
            adapter_name: self.adapter_name(),
            fixture,
            request,
            profile,
            plan_summary: report.plan_summary,
            report: &report,
            runtime_policy,
            target_statuses,
        }))
    }

    fn capture_snapshot(
        &self,
        runtime: &Self::Runtime,
        fixture: &ScenarioFixture<Self::Fixture>,
        request: &ExecutionRequest<Self::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<SnapshotRecord<Self::TargetId>, Self::Error> {
        let runtime = runtime.runtime()?;
        SignalHarnessBridge::snapshot_record(
            self.adapter_name(),
            runtime,
            fixture,
            request,
            profile,
        )
    }
}
