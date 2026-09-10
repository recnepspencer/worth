use std::sync::Arc;

use super::step_contract_admission::WorthQueryAdmittedManagedStepContract;
use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    provider_anchor::WorthQueryGraphProviderAnchor, WorthQueryGraphProviderMemoryArena,
    WorthQueryGraphProviderStepArtifactContext, WorthQueryGraphProviderStepCompletion,
    WorthQueryOwnedGraphProviderExecution, WorthQueryProviderExecutionInvocation,
};
use crate::domain_computation::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryGraphProviderCall,
    WorthQueryGraphProviderCallKind, WorthQueryGraphProviderExecution, WorthQueryGraphProviderStep,
    WorthQueryGraphProviderStepReport, WorthQueryGraphProviderStepRetainedEvidence,
    WorthQueryGraphReadMaterial, WorthQueryGraphReadStreamAccumulator,
    WorthQueryProviderWorkReport,
};

pub(super) struct WorthQueryManagedGraphExecution {
    pub(super) call: WorthQueryGraphProviderCall,
    pub(super) execution: WorthQueryOwnedGraphProviderExecution,
    pub(super) anchor: Arc<WorthQueryGraphProviderAnchor>,
    pub(super) contract: WorthQueryAdmittedManagedStepContract,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
    pub(super) completed_work_units: u64,
    pub(super) applied_effect_count: u64,
    pub(super) peak_scratch_bytes: u64,
    pub(super) retained_bytes: u64,
    pub(super) output_retained_bytes: u64,
    pub(super) projection: Option<WorthQueryGraphReadStreamAccumulator>,
    pub(super) artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    pub(super) produced_artifact_count: usize,
    pub(super) retained_artifact_count: usize,
    pub(super) disposed_artifact_count: usize,
    last_checkpoint_available: bool,
    last_retained: WorthQueryGraphProviderStepRetainedEvidence,
}

pub(super) struct WorthQueryManagedGraphExecutionStartParts {
    pub(super) call: WorthQueryGraphProviderCall,
    pub(super) execution: Box<dyn WorthQueryGraphProviderExecution>,
    pub(super) anchor: Arc<WorthQueryGraphProviderAnchor>,
    pub(super) contract: WorthQueryAdmittedManagedStepContract,
    pub(super) artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
}

pub(super) struct WorthQueryRestoredManagedGraphExecutionParts {
    pub(super) call: WorthQueryGraphProviderCall,
    pub(super) execution: Box<dyn WorthQueryGraphProviderExecution>,
    pub(super) anchor: Arc<WorthQueryGraphProviderAnchor>,
    pub(super) contract: WorthQueryAdmittedManagedStepContract,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
    pub(super) completed_work_units: u64,
    pub(super) applied_effect_count: u64,
    pub(super) peak_scratch_bytes: u64,
    pub(super) retained_bytes: u64,
    pub(super) projection: Option<WorthQueryGraphReadStreamAccumulator>,
    pub(super) artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    pub(super) produced_artifact_count: usize,
    pub(super) retained_artifact_count: usize,
    pub(super) disposed_artifact_count: usize,
}

pub(super) enum WorthQueryManagedProviderStep {
    Continue(WorthQueryManagedProviderStepEvidence),
    Complete(WorthQueryManagedProviderStepEvidence),
    Failed(WorthQueryManagedProviderStepEvidence),
}

pub(super) struct WorthQueryManagedProviderStepEvidence {
    admission: super::provider_step_admission::WorthQueryAdmittedProviderStep,
    report: WorthQueryGraphProviderStepReport,
}

impl WorthQueryManagedProviderStepEvidence {
    fn new(
        admission: super::provider_step_admission::WorthQueryAdmittedProviderStep,
        report: WorthQueryGraphProviderStepReport,
    ) -> Self {
        Self { admission, report }
    }

    pub(super) fn into_report(self) -> WorthQueryGraphProviderStepReport {
        let _ = self.admission.observation();
        self.report
    }
}

impl WorthQueryManagedGraphExecution {
    pub(super) fn new(parts: WorthQueryManagedGraphExecutionStartParts) -> Self {
        let projection = (parts.call.kind() == WorthQueryGraphProviderCallKind::Project)
            .then(|| WorthQueryGraphReadStreamAccumulator::new(&parts.call));
        Self {
            call: parts.call,
            execution: WorthQueryOwnedGraphProviderExecution::new(parts.execution),
            anchor: parts.anchor,
            contract: parts.contract,
            memory: parts.memory,
            completed_work_units: 0,
            applied_effect_count: 0,
            peak_scratch_bytes: 0,
            retained_bytes: 0,
            output_retained_bytes: 0,
            projection,
            artifact_context: parts.artifact_context,
            produced_artifact_count: 0,
            retained_artifact_count: 0,
            disposed_artifact_count: 0,
            last_checkpoint_available: false,
            last_retained: WorthQueryGraphProviderStepRetainedEvidence::default(),
        }
    }

    pub(super) fn restored(parts: WorthQueryRestoredManagedGraphExecutionParts) -> Self {
        let output_retained_bytes = parts
            .projection
            .as_ref()
            .map(|projection| u64::try_from(projection.retained_bytes()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        Self {
            call: parts.call,
            execution: WorthQueryOwnedGraphProviderExecution::new(parts.execution),
            anchor: parts.anchor,
            contract: parts.contract,
            memory: parts.memory,
            completed_work_units: parts.completed_work_units,
            applied_effect_count: parts.applied_effect_count,
            peak_scratch_bytes: parts.peak_scratch_bytes,
            retained_bytes: parts.retained_bytes,
            output_retained_bytes,
            projection: parts.projection,
            artifact_context: parts.artifact_context,
            produced_artifact_count: parts.produced_artifact_count,
            retained_artifact_count: parts.retained_artifact_count,
            disposed_artifact_count: parts.disposed_artifact_count,
            last_checkpoint_available: false,
            last_retained: WorthQueryGraphProviderStepRetainedEvidence::default(),
        }
    }

    pub(super) fn admit_provider_step(
        &self,
        observation: super::WorthQueryManagedSafePointObservation,
    ) -> super::provider_step_admission::WorthQueryProviderStepAdmissionOutcome {
        super::provider_step_admission::admit_provider_step(
            self.call.kind(),
            self.contract.installed(),
            observation,
        )
    }

    pub(super) fn advance_provider(
        &mut self,
        admission: super::provider_step_admission::WorthQueryAdmittedProviderStep,
    ) -> WorthQueryManagedProviderStep {
        let managed_retained_bytes = if self.call.kind() == WorthQueryGraphProviderCallKind::Project
        {
            self.output_retained_bytes
                .saturating_add(
                    u64::try_from(
                        WorthQueryGraphReadStreamAccumulator::chunk_node_allocation_bytes(),
                    )
                    .unwrap_or(u64::MAX),
                )
                .saturating_add(
                    u64::try_from(WorthQueryGraphReadStreamAccumulator::stream_allocation_bytes())
                        .unwrap_or(u64::MAX),
                )
        } else {
            0
        };
        let mut step = WorthQueryGraphProviderStep::new(
            self.call.kind(),
            self.contract.installed(),
            self.artifact_context.clone(),
            self.memory.clone(),
            managed_retained_bytes,
        );
        let disposition = match self.execution.advance(&mut step) {
            WorthQueryProviderExecutionInvocation::Returned(Ok(disposition)) => disposition,
            WorthQueryProviderExecutionInvocation::Returned(Err(failure)) => {
                return WorthQueryManagedProviderStep::Failed(
                    WorthQueryManagedProviderStepEvidence::new(
                        admission,
                        step.finish_rejected(failure),
                    ),
                )
            }
            WorthQueryProviderExecutionInvocation::Panicked => {
                return WorthQueryManagedProviderStep::Failed(
                    WorthQueryManagedProviderStepEvidence::new(admission, step.finish_panicked()),
                )
            }
        };
        let report = match step.finish(disposition) {
            Ok(report) => report,
            Err((_denial, report)) => {
                return WorthQueryManagedProviderStep::Failed(
                    WorthQueryManagedProviderStepEvidence::new(admission, report),
                )
            }
        };
        let completion = report.completion();
        let evidence = WorthQueryManagedProviderStepEvidence::new(admission, report);
        match completion {
            WorthQueryGraphProviderStepCompletion::Continue => {
                WorthQueryManagedProviderStep::Continue(evidence)
            }
            WorthQueryGraphProviderStepCompletion::Complete => {
                WorthQueryManagedProviderStep::Complete(evidence)
            }
            WorthQueryGraphProviderStepCompletion::Failed => {
                WorthQueryManagedProviderStep::Failed(evidence)
            }
        }
    }

    pub(super) fn admit_report(&mut self, report: &mut WorthQueryGraphProviderStepReport) {
        self.completed_work_units = self
            .completed_work_units
            .saturating_add(report.completed_work_units());
        self.applied_effect_count = self
            .applied_effect_count
            .saturating_add(report.applied_effect_count());
        self.peak_scratch_bytes = self.peak_scratch_bytes.max(report.peak_scratch_bytes());
        self.retained_bytes = report
            .retained_bytes()
            .saturating_add(self.output_retained_bytes);
        self.last_checkpoint_available = report.checkpoint_available();
        self.last_retained = report.retained_evidence();
        self.last_retained
            .retain_prior_output_bytes(self.output_retained_bytes);
        let artifacts = report.artifact_evidence();
        self.produced_artifact_count = self
            .produced_artifact_count
            .saturating_add(artifacts.produced_artifact_count());
        self.retained_artifact_count = artifacts.retained_artifact_count();
        self.disposed_artifact_count = self
            .disposed_artifact_count
            .saturating_add(artifacts.disposed_artifact_count());
    }

    pub(super) fn yield_safe_point(
        &self,
        observation: super::WorthQueryManagedSafePointObservation,
    ) -> super::yield_eligibility::WorthQueryManagedYieldSafePoint {
        assert_eq!(
            observation.signal_state(),
            worth_runtime_bridge::facade::BridgeExecutionSafePointSignalState::Active,
            "yield safe-point authority requires an active Signal attempt",
        );
        assert_eq!(
            observation.queue_depth(),
            0,
            "yield safe-point authority requires a drained result queue",
        );
        assert!(
            self.applied_effect_count == 0
                || self.contract.installed().partial_effects_may_remain(),
            "yield safe-point authority requires the installed partial-effect posture",
        );
        super::yield_eligibility::WorthQueryManagedYieldSafePoint::new(
            self.last_checkpoint_available,
            self.last_retained,
        )
    }

    pub(super) fn provider_call_identity(&self) -> &str {
        self.call.call_identity()
    }

    pub(super) fn projection_chunk_fits_ceiling(&self) -> bool {
        self.retained_bytes
            .saturating_add(
                u64::try_from(WorthQueryGraphReadStreamAccumulator::chunk_node_allocation_bytes())
                    .unwrap_or(u64::MAX),
            )
            .saturating_add(
                u64::try_from(WorthQueryGraphReadStreamAccumulator::stream_allocation_bytes())
                    .unwrap_or(u64::MAX),
            )
            <= self.contract.installed().retained_bytes_ceiling()
    }

    pub(super) fn retain_projection_chunk(
        &mut self,
        material: WorthQueryGraphReadMaterial,
    ) -> Option<(usize, usize)> {
        let projection_bytes = material.owned_allocation_capacity_bytes();
        let retained_bytes = self
            .projection
            .as_mut()
            .expect("only projection executions admit projection chunks")
            .admit_chunk(material);
        let additional_bytes = retained_bytes.checked_sub(projection_bytes)?;
        if !self.last_retained.transfer_projection_to_output(
            u64::try_from(projection_bytes).unwrap_or(u64::MAX),
            u64::try_from(additional_bytes).unwrap_or(u64::MAX),
        ) {
            return None;
        }
        self.output_retained_bytes = self
            .output_retained_bytes
            .saturating_add(u64::try_from(retained_bytes).unwrap_or(u64::MAX));
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(u64::try_from(additional_bytes).unwrap_or(u64::MAX));
        Some((projection_bytes, additional_bytes))
    }

    pub(super) fn release_projection_chunk(&mut self, retained_bytes: usize) -> bool {
        let retained_bytes = u64::try_from(retained_bytes).unwrap_or(u64::MAX);
        let Some(remaining) = self.retained_bytes.checked_sub(retained_bytes) else {
            return false;
        };
        if !self.last_retained.release_projection_bytes(retained_bytes) {
            return false;
        }
        self.retained_bytes = remaining;
        true
    }

    pub(super) fn seal_completion(
        &mut self,
        report: &WorthQueryGraphProviderStepReport,
    ) -> Result<(WorthQueryBoundGraphExecutionReceipt, usize, usize), ()> {
        let stream_allocation_bytes =
            if self.call.kind() == WorthQueryGraphProviderCallKind::Project {
                WorthQueryGraphReadStreamAccumulator::stream_allocation_bytes()
            } else {
                0
            };
        let stream_allocation_bytes_u64 =
            u64::try_from(stream_allocation_bytes).unwrap_or(u64::MAX);
        if self
            .retained_bytes
            .saturating_add(stream_allocation_bytes_u64)
            > self.contract.installed().retained_bytes_ceiling()
        {
            return Err(());
        }
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(stream_allocation_bytes_u64);
        self.output_retained_bytes = self
            .output_retained_bytes
            .saturating_add(stream_allocation_bytes_u64);
        let provider_receipt = Arc::<str>::from(report.provider_receipt().ok_or(())?);
        let work = WorthQueryProviderWorkReport::new(
            self.completed_work_units,
            self.applied_effect_count,
            usize::try_from(self.peak_scratch_bytes).unwrap_or(usize::MAX),
            usize::try_from(self.retained_bytes).unwrap_or(usize::MAX),
        )
        .with_output_retention(usize::try_from(self.output_retained_bytes).unwrap_or(usize::MAX))
        .with_artifact_disposition(
            self.produced_artifact_count,
            self.retained_artifact_count,
            self.disposed_artifact_count,
        )
        .ok_or(())?;
        let receipt = if self.call.kind() == WorthQueryGraphProviderCallKind::Project {
            let stream = self.projection.take().ok_or(())?.finish(&self.call);
            self.call
                .streamed(provider_receipt, stream, work)
                .map_err(|_| ())?
        } else {
            self.call.completed(provider_receipt, work)
        };
        let receipt = self.call.admit_receipt(receipt).map_err(|_| ())?;
        Ok((
            receipt,
            usize::try_from(self.output_retained_bytes).unwrap_or(usize::MAX),
            stream_allocation_bytes,
        ))
    }

    pub(super) fn output_retained_bytes(&self) -> usize {
        usize::try_from(self.output_retained_bytes).unwrap_or(usize::MAX)
    }
}
