use crate::capture::{RunRecord, SnapshotRecord};
use crate::comparison::{ComparisonProfile, ComparisonRecord};
use crate::scenario::{
    CaptureMask, ExecutionProfile, ExecutionRequest, MutationBatch, ScenarioFixture,
};

use super::super::adapter::HarnessAdapter;
use super::super::capability::AdapterSupport;
use super::bundles::HarnessCoreBundle;
use super::error::HarnessError;

pub struct HarnessRunner<A> {
    pub(super) adapter: A,
}

impl<A> HarnessRunner<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

pub(super) struct LoadedHarnessRun<A: HarnessAdapter> {
    pub(super) runtime: A::Runtime,
    pub(super) core: HarnessCoreBundle<A::TargetId>,
}

impl<A> HarnessRunner<A>
where
    A: HarnessAdapter,
    A::TargetId: PartialEq,
{
    pub(super) fn capture_request(
        &self,
        request: &ExecutionRequest<A::TargetId>,
    ) -> ExecutionRequest<A::TargetId> {
        let mut filtered = request.clone();
        if let Some(included) = &request.capture.target_policy.included_targets {
            filtered.targets = request
                .targets
                .iter()
                .filter(|target| included.iter().any(|candidate| candidate == *target))
                .cloned()
                .collect();
        }
        filtered
    }

    fn minimal_run_record(
        &self,
        mut run: RunRecord<A::TargetId>,
        capture_mask: &CaptureMask,
    ) -> RunRecord<A::TargetId> {
        if !capture_mask.run_summary {
            run.summary = serde_json::json!({});
        }
        if !capture_mask.attachments {
            run.attachments.clear();
        }
        run
    }

    pub(super) fn execute_loaded(
        &self,
        fixture: &ScenarioFixture<A::Fixture>,
        mutation_batch: Option<&MutationBatch<A::Mutation>>,
        request: &ExecutionRequest<A::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<LoadedHarnessRun<A>, HarnessError<A::Error>> {
        self.validate_execution_profile(request, profile)?;
        let mut runtime = self.prepare_loaded_runtime(fixture, profile)?;
        let scenario = self.adapter.scenario_record(fixture);
        let capture_request = self.capture_request(request);
        let pre_snapshot = self.capture_optional_snapshot(
            &runtime,
            fixture,
            &capture_request,
            profile,
            request.capture.mask.pre_snapshot,
        )?;
        let run = self.apply_mutation_and_execute(
            &mut runtime,
            fixture,
            mutation_batch,
            request,
            profile,
        )?;
        let post_snapshot = self.capture_optional_snapshot(
            &runtime,
            fixture,
            &capture_request,
            profile,
            request.capture.mask.post_snapshot,
        )?;

        Ok(LoadedHarnessRun {
            runtime,
            core: HarnessCoreBundle {
                scenario,
                pre_snapshot,
                run,
                post_snapshot,
            },
        })
    }

    fn prepare_loaded_runtime(
        &self,
        fixture: &ScenarioFixture<A::Fixture>,
        profile: &ExecutionProfile,
    ) -> Result<A::Runtime, HarnessError<A::Error>> {
        let mut runtime = self
            .adapter
            .create_runtime()
            .map_err(HarnessError::Adapter)?;
        self.adapter
            .prepare_runtime(&mut runtime, profile)
            .map_err(HarnessError::Adapter)?;
        self.adapter
            .load_fixture(&mut runtime, fixture)
            .map_err(HarnessError::Adapter)?;
        Ok(runtime)
    }

    fn capture_optional_snapshot(
        &self,
        runtime: &A::Runtime,
        fixture: &ScenarioFixture<A::Fixture>,
        request: &ExecutionRequest<A::TargetId>,
        profile: &ExecutionProfile,
        enabled: bool,
    ) -> Result<Option<SnapshotRecord<A::TargetId>>, HarnessError<A::Error>> {
        if enabled {
            Ok(Some(
                self.adapter
                    .capture_snapshot(runtime, fixture, request, profile)
                    .map_err(HarnessError::Adapter)?,
            ))
        } else {
            Ok(None)
        }
    }

    fn apply_mutation_and_execute(
        &self,
        runtime: &mut A::Runtime,
        fixture: &ScenarioFixture<A::Fixture>,
        mutation_batch: Option<&MutationBatch<A::Mutation>>,
        request: &ExecutionRequest<A::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<RunRecord<A::TargetId>, HarnessError<A::Error>> {
        if let Some(batch) = mutation_batch {
            self.adapter
                .apply_mutation_batch(runtime, batch)
                .map_err(HarnessError::Adapter)?;
        }
        let run = self
            .adapter
            .execute(runtime, fixture, request, profile)
            .map_err(HarnessError::Adapter)?;
        Ok(self.minimal_run_record(run, &request.capture.mask))
    }

    fn validate_execution_profile(
        &self,
        request: &ExecutionRequest<A::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<(), HarnessError<A::Error>> {
        let capabilities = self.adapter.capabilities();
        if !capabilities.supports_execution_mode(profile.execution_mode) {
            return Err(HarnessError::UnsupportedExecutionMode(
                profile.execution_mode,
            ));
        }
        if !capabilities.supports_diagnostics_level(profile.diagnostics_level) {
            return Err(HarnessError::UnsupportedDiagnosticsLevel(
                profile.diagnostics_level,
            ));
        }
        if !capabilities.supports_capture_depth(profile.capture_depth) {
            return Err(HarnessError::UnsupportedCaptureDepth(profile.capture_depth));
        }
        if !capabilities.supports_clock_domain(profile.clock_domain) {
            return Err(HarnessError::UnsupportedClockDomain(profile.clock_domain));
        }
        if let Some(phase) = profile
            .workload
            .as_ref()
            .and_then(|workload| workload.phase)
            && !capabilities.supports_execution_phase(phase)
        {
            return Err(HarnessError::UnsupportedExecutionPhase(phase));
        }
        if let Some(phase) = request
            .feed_batch
            .as_ref()
            .and_then(|feed_batch| feed_batch.phase)
            && !capabilities.supports_execution_phase(phase)
        {
            return Err(HarnessError::UnsupportedExecutionPhase(phase));
        }
        if (profile.workload.is_some() || profile.work_budget.is_some())
            && !matches!(
                capabilities.workload_budget_support,
                AdapterSupport::Supported
            )
        {
            return Err(HarnessError::UnsupportedWorkBudget);
        }
        Ok(())
    }

    pub fn execute_core(
        &self,
        fixture: &ScenarioFixture<A::Fixture>,
        mutation_batch: Option<&MutationBatch<A::Mutation>>,
        request: &ExecutionRequest<A::TargetId>,
        profile: &ExecutionProfile,
    ) -> Result<HarnessCoreBundle<A::TargetId>, HarnessError<A::Error>> {
        Ok(self
            .execute_loaded(fixture, mutation_batch, request, profile)?
            .core)
    }

    pub fn compare_runs(
        &self,
        left: &RunRecord<A::TargetId>,
        right: &RunRecord<A::TargetId>,
        profile: &ComparisonProfile,
    ) -> Result<ComparisonRecord, HarnessError<A::Error>>
    where
        A::TargetId: std::fmt::Debug + PartialEq,
    {
        let capabilities = self.adapter.capabilities();
        if !capabilities.supports_comparison_mode(profile.mode) {
            return Err(HarnessError::UnsupportedComparisonMode(profile.mode));
        }
        Ok(crate::comparison::compare_run_records(left, right, profile))
    }
}
