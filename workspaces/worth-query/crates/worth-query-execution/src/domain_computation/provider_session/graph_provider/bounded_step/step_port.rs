use worth_query_installation::facade::WorthQueryInstalledBoundedStepContract;

use super::artifact_evidence::WorthQueryGraphProviderStepArtifacts;
use super::memory::allocate_scratch_bytes;
use super::step_budget::WorthQueryGraphProviderStepBudget;
use super::step_state::WorthQueryGraphProviderStepState;
use super::{
    WorthQueryGraphProviderMemoryArena, WorthQueryGraphProviderRetainedMemory,
    WorthQueryGraphProviderStepDenial, WorthQueryGraphProviderStepDenialKind,
    WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderStepDispositionKind,
    WorthQueryGraphProviderStepFailureEvidence, WorthQueryGraphProviderStepReport,
    WorthQueryGraphProviderStepReportParts, WorthQueryGraphProviderStepRetainedEvidence,
};
use crate::domain_computation::{
    WorthQueryArtifactProductionEvidence, WorthQueryArtifactProviderResource,
    WorthQueryGraphProviderCallKind, WorthQueryGraphProviderFailure, WorthQueryGraphReadMaterial,
    WorthQueryMoveOnlyArtifactHandle,
};

pub struct WorthQueryGraphProviderStep {
    call_kind: WorthQueryGraphProviderCallKind,
    budget: WorthQueryGraphProviderStepBudget,
    attempted_effect_count: u64,
    applied_effect_count: u64,
    projection: Option<WorthQueryGraphReadMaterial>,
    artifacts: WorthQueryGraphProviderStepArtifacts,
    state: WorthQueryGraphProviderStepState,
    partial_effects_may_remain: bool,
    checkpoint_available: bool,
    memory: WorthQueryGraphProviderMemoryArena,
}

impl WorthQueryGraphProviderStep {
    pub(crate) fn new(
        call_kind: WorthQueryGraphProviderCallKind,
        contract: &WorthQueryInstalledBoundedStepContract,
        artifact_context: Option<super::WorthQueryGraphProviderStepArtifactContext>,
        memory: WorthQueryGraphProviderMemoryArena,
        managed_retained_bytes: u64,
    ) -> Self {
        Self {
            call_kind,
            budget: WorthQueryGraphProviderStepBudget::new(contract, managed_retained_bytes),
            attempted_effect_count: 0,
            applied_effect_count: 0,
            projection: None,
            artifacts: WorthQueryGraphProviderStepArtifacts::new(artifact_context),
            state: WorthQueryGraphProviderStepState::default(),
            partial_effects_may_remain: contract.partial_effects_may_remain(),
            checkpoint_available: false,
            memory,
        }
    }

    pub const fn call_kind(&self) -> WorthQueryGraphProviderCallKind {
        self.call_kind
    }

    pub const fn remaining_work_units(&self) -> u64 {
        self.budget.remaining_work_units()
    }

    pub fn perform_work_unit<Output>(
        &mut self,
        work: impl FnOnce() -> Result<Output, WorthQueryGraphProviderFailure>,
    ) -> Result<Output, WorthQueryGraphProviderFailure> {
        self.state
            .ensure_active()
            .map_err(step_denial_as_provider_failure)?;
        let admission = self.budget.admit_work_unit();
        self.state
            .admit(admission)
            .map_err(step_denial_as_provider_failure)?;
        let output = match work() {
            Ok(output) => output,
            Err(failure) => return Err(self.state.reject_provider(failure)),
        };
        self.budget.complete_work_unit();
        Ok(output)
    }

    pub fn apply_effect<Output>(
        &mut self,
        effect: impl FnOnce() -> Result<Output, WorthQueryGraphProviderFailure>,
    ) -> Result<Output, WorthQueryGraphProviderFailure> {
        self.state
            .ensure_active()
            .map_err(step_denial_as_provider_failure)?;
        if self.call_kind != WorthQueryGraphProviderCallKind::TouchEffect {
            return self
                .deny(WorthQueryGraphProviderStepDenial::new(
                    WorthQueryGraphProviderStepDenialKind::UnexpectedEffect,
                    "only a graph effect call may record an applied effect",
                ))
                .map_err(step_denial_as_provider_failure);
        }
        if !self.partial_effects_may_remain {
            return self
                .deny(WorthQueryGraphProviderStepDenial::new(
                    WorthQueryGraphProviderStepDenialKind::EffectPostureDenied,
                    "the installed bounded-step contract is effect-free",
                ))
                .map_err(step_denial_as_provider_failure);
        }
        let admission = self.budget.admit_work_unit();
        self.state
            .admit(admission)
            .map_err(step_denial_as_provider_failure)?;
        self.attempted_effect_count = self.attempted_effect_count.saturating_add(1);
        let output = match effect() {
            Ok(output) => output,
            Err(failure) => return Err(self.state.reject_provider(failure)),
        };
        self.budget.complete_work_unit();
        self.applied_effect_count = self.applied_effect_count.saturating_add(1);
        Ok(output)
    }

    pub fn emit_projection_chunk(
        &mut self,
        material: WorthQueryGraphReadMaterial,
    ) -> Result<(), WorthQueryGraphProviderStepDenial> {
        self.state.ensure_active()?;
        if self.call_kind != WorthQueryGraphProviderCallKind::Project {
            return self.deny(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::UnexpectedProjection,
                "only a graph projection call may emit projection material",
            ));
        }
        if self.projection.is_some() {
            return self.deny(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::MultipleProjectionChunks,
                "one bounded provider step may emit at most one projection chunk",
            ));
        }
        let admission = self.budget.validate_chunk_width(material.rows().len());
        self.state.admit(admission)?;
        self.projection = Some(material);
        Ok(())
    }

    pub fn with_scratch_bytes<Output>(
        &mut self,
        byte_count: usize,
        operation: impl FnOnce(&mut [u8]) -> Result<Output, WorthQueryGraphProviderFailure>,
    ) -> Result<Output, WorthQueryGraphProviderFailure> {
        self.state
            .ensure_active()
            .map_err(step_denial_as_provider_failure)?;
        let requested = u64::try_from(byte_count).unwrap_or(u64::MAX);
        let validation = self.budget.validate_scratch(requested);
        self.state
            .admit(validation)
            .map_err(step_denial_as_provider_failure)?;
        let mut scratch: Vec<u8> = self
            .state
            .admit(allocate_scratch_bytes(byte_count))
            .map_err(step_denial_as_provider_failure)?;
        let actual = u64::try_from(scratch.capacity()).unwrap_or(u64::MAX);
        let admission = self.budget.admit_scratch(actual);
        self.state
            .admit(admission)
            .map_err(step_denial_as_provider_failure)?;
        match operation(&mut scratch) {
            Ok(output) => Ok(output),
            Err(failure) => Err(self.state.reject_provider(failure)),
        }
    }

    pub fn retain_bytes(
        &mut self,
        byte_count: usize,
    ) -> Result<WorthQueryGraphProviderRetainedMemory, WorthQueryGraphProviderStepDenial> {
        self.state.ensure_active()?;
        self.state.admit(self.memory.retain_bytes(byte_count))
    }

    pub fn produce_artifact<R: WorthQueryArtifactProviderResource>(
        &mut self,
        evidence: WorthQueryArtifactProductionEvidence,
        resource: R,
    ) -> Result<WorthQueryMoveOnlyArtifactHandle, WorthQueryGraphProviderStepDenial> {
        self.state.ensure_active()?;
        let production = self
            .artifacts
            .produce(evidence, resource)
            .map_err(|denial| {
                WorthQueryGraphProviderStepDenial::new(
                    WorthQueryGraphProviderStepDenialKind::ArtifactAdmissionDenied,
                    denial.detail(),
                )
            });
        self.state.admit(production)
    }

    pub fn record_checkpoint_available(&mut self) -> Result<(), WorthQueryGraphProviderStepDenial> {
        self.state.ensure_active()?;
        if self.checkpoint_available {
            return self.deny(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::MultipleCheckpoints,
                "one bounded provider step may record checkpoint availability once",
            ));
        }
        self.checkpoint_available = true;
        Ok(())
    }

    pub(crate) fn finish(
        mut self,
        disposition: WorthQueryGraphProviderStepDisposition,
    ) -> Result<
        WorthQueryGraphProviderStepReport,
        (
            WorthQueryGraphProviderStepDenial,
            WorthQueryGraphProviderStepReport,
        ),
    > {
        let artifacts = self.artifacts.finish();
        let memory = self.memory.snapshot();
        let projection_retained_bytes = self.projection_retained_bytes();
        self.admit_retained_evidence(
            memory.retained_bytes(),
            projection_retained_bytes,
            artifacts.retained_bytes(),
        );
        self.validate_completion(&disposition);
        if let Some((denial, failure)) = self.latched_failure() {
            let parts = self.into_report_parts(artifacts, memory, projection_retained_bytes);
            return Err((
                denial,
                WorthQueryGraphProviderStepReport::failed(parts, failure),
            ));
        }
        let parts = self.into_report_parts(artifacts, memory, projection_retained_bytes);
        Ok(WorthQueryGraphProviderStepReport::from_disposition(
            disposition,
            parts,
        ))
    }

    pub(crate) fn finish_rejected(
        self,
        failure: WorthQueryGraphProviderFailure,
    ) -> WorthQueryGraphProviderStepReport {
        let evidence = WorthQueryGraphProviderStepFailureEvidence::rejected(
            failure,
            self.state.governed_denial().cloned(),
            self.state.provider_failure().cloned(),
        );
        let artifacts = self.artifacts.finish();
        self.finish_failed_with_artifacts(artifacts, evidence)
    }

    pub(crate) fn finish_panicked(self) -> WorthQueryGraphProviderStepReport {
        let evidence = WorthQueryGraphProviderStepFailureEvidence::panicked(
            self.state.governed_denial().cloned(),
            self.state.provider_failure().cloned(),
        );
        let artifacts = self.artifacts.finish();
        self.finish_failed_with_artifacts(artifacts, evidence)
    }

    fn finish_failed_with_artifacts(
        self,
        artifacts: super::WorthQueryGraphProviderStepArtifactEvidence,
        failure: WorthQueryGraphProviderStepFailureEvidence,
    ) -> WorthQueryGraphProviderStepReport {
        let projection_retained_bytes = self.projection_retained_bytes();
        let memory = self.memory.snapshot();
        let parts = self.into_report_parts(artifacts, memory, projection_retained_bytes);
        WorthQueryGraphProviderStepReport::failed(parts, failure)
    }

    fn admit_retained_evidence(
        &mut self,
        provider_bytes: u64,
        projection_bytes: usize,
        artifact_bytes: usize,
    ) {
        self.admit_retained_component(provider_bytes);
        self.admit_retained_component(u64::try_from(projection_bytes).unwrap_or(u64::MAX));
        self.admit_retained_component(u64::try_from(artifact_bytes).unwrap_or(u64::MAX));
    }

    fn admit_retained_component(&mut self, retained_bytes: u64) {
        if self.state.has_failure() {
            return;
        }
        let admission = self.budget.admit_retained_component(retained_bytes);
        let _ = self.state.admit(admission);
    }

    fn validate_completion(&mut self, disposition: &WorthQueryGraphProviderStepDisposition) {
        if !self.state.has_failure()
            && disposition.kind() == WorthQueryGraphProviderStepDispositionKind::Complete
            && self.call_kind == WorthQueryGraphProviderCallKind::Project
            && self.projection.is_none()
        {
            self.state.deny(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::MissingProjectionChunk,
                "a completed graph projection must emit one explicit projection chunk",
            ));
        }
        if !self.state.has_failure() && self.budget.completed_work_units() == 0 {
            self.state.deny(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::NoProgress,
                "provider step produced no successfully completed governed work",
            ));
        }
    }

    fn latched_failure(
        &self,
    ) -> Option<(
        WorthQueryGraphProviderStepDenial,
        WorthQueryGraphProviderStepFailureEvidence,
    )> {
        self.state.has_failure().then(|| {
            let governed_denial = self.state.governed_denial().cloned();
            let denial = governed_denial.clone().unwrap_or_else(|| {
                WorthQueryGraphProviderStepDenial::new(
                    WorthQueryGraphProviderStepDenialKind::ProviderFailureLatched,
                    "provider step retained a rejected governed operation",
                )
            });
            let failure = WorthQueryGraphProviderStepFailureEvidence::returned(
                governed_denial,
                self.state.provider_failure().cloned(),
            );
            (denial, failure)
        })
    }

    fn projection_retained_bytes(&self) -> usize {
        self.projection.as_ref().map_or(
            0,
            WorthQueryGraphReadMaterial::owned_allocation_capacity_bytes,
        )
    }

    fn into_report_parts(
        self,
        artifacts: super::WorthQueryGraphProviderStepArtifactEvidence,
        memory: super::WorthQueryGraphProviderMemorySnapshot,
        projection_retained_bytes: usize,
    ) -> WorthQueryGraphProviderStepReportParts {
        WorthQueryGraphProviderStepReportParts {
            completed_work_units: self.budget.completed_work_units(),
            attempted_effect_count: self.attempted_effect_count,
            applied_effect_count: self.applied_effect_count,
            peak_scratch_bytes: self.budget.peak_scratch_bytes(),
            retained: WorthQueryGraphProviderStepRetainedEvidence::new(
                memory,
                projection_retained_bytes,
                artifacts.retained_bytes(),
            ),
            projection: self.projection,
            artifacts,
            checkpoint_available: self.checkpoint_available,
        }
    }

    fn deny<Output>(
        &mut self,
        denial: WorthQueryGraphProviderStepDenial,
    ) -> Result<Output, WorthQueryGraphProviderStepDenial> {
        Err(self.state.deny(denial))
    }
}

fn step_denial_as_provider_failure(
    denial: WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
