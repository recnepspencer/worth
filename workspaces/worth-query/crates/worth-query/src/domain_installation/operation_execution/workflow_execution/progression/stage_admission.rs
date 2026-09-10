use crate::basis_lifecycle::BasisOperationLane;

use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind, WorthQueryWorkflowRun,
    WorthQueryWorkflowValue,
};

pub(crate) struct WorthQueryWorkflowStageRuntimeAdmission {
    _private: (),
}

pub(crate) struct WorthQueryAdmittedWorkflowStage {
    pub(super) stage: worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    pub(super) counters_before: super::WorthQueryWorkflowRunCounters,
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(super) fn admit_stage(
        &mut self,
        stage_identity: &str,
        input: &WorthQueryWorkflowValue,
        _runtime_admission: WorthQueryWorkflowStageRuntimeAdmission,
    ) -> Result<WorthQueryAdmittedWorkflowStage, WorthQueryWorkflowAdvanceDenial> {
        let admitted = self.admit_stage_readiness(stage_identity)?;
        let stage = &admitted.stage;
        if !input.satisfies(&stage.semantics().input) {
            return Err(
                self.stage_admission_denial(WorthQueryWorkflowAdvanceDenialKind::InputContract)
            );
        }
        if let Err(denial) = self.validate_artifact_input(stage, input) {
            return Err(self.stage_admission_denial(
                WorthQueryWorkflowAdvanceDenialKind::ArtifactCarriage(denial),
            ));
        }
        Ok(admitted)
    }

    pub(super) fn admit_artifact_stage(
        &mut self,
        stage_identity: &str,
        _runtime_admission: WorthQueryWorkflowStageRuntimeAdmission,
    ) -> Result<WorthQueryAdmittedWorkflowStage, WorthQueryWorkflowAdvanceDenial> {
        let admitted = self.admit_stage_readiness(stage_identity)?;
        if !matches!(
            admitted.stage.semantics().input,
            worth_query_installation::facade::WorthQueryWorkflowValueContract::InstalledArtifact(_)
        ) {
            return Err(
                self.stage_admission_denial(WorthQueryWorkflowAdvanceDenialKind::InputContract)
            );
        }
        Ok(admitted)
    }

    fn admit_stage_readiness(
        &mut self,
        stage_identity: &str,
    ) -> Result<WorthQueryAdmittedWorkflowStage, WorthQueryWorkflowAdvanceDenial> {
        let counters_before = self.counters;
        let stage = self.pending_stage(stage_identity)?;
        self.validate_stage_predecessors(&stage)?;
        self.validate_stage_capabilities(&stage)?;
        self.validate_stage_domains(&stage)?;
        Ok(WorthQueryAdmittedWorkflowStage {
            stage,
            counters_before,
        })
    }

    pub(super) fn validate_artifact_input(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &WorthQueryWorkflowValue,
    ) -> Result<(), crate::domain_installation::WorthQueryArtifactDenial> {
        let worth_query_installation::facade::WorthQueryWorkflowValueContract::InstalledArtifact(_) =
            &stage.semantics().input
        else {
            return Ok(());
        };
        let WorthQueryWorkflowValue::TransferredArtifact(handle) = input else {
            return Err(crate::domain_installation::WorthQueryArtifactDenial::new(
                crate::domain_installation::WorthQueryArtifactDenialKind::StageMismatch,
                None,
                "artifact workflow input must come from an admitted stage transfer",
            ));
        };
        let admission = self
            .managed_run()
            .artifacts()
            .input_validation_admission(stage.identity())?;
        handle.validate_input(&admission)
    }

    pub(super) fn admit_stage_runtime_authority(
        &mut self,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowStageRuntimeAdmission, WorthQueryWorkflowAdvanceDenial> {
        self.counters.runtime_authority_checks += 1;
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                std::sync::Arc::clone(self.bound.operation().domain_authority()),
            );
        workspace
            .validate_installed_domain_witness::<D>(&witness)
            .map_err(|denial| {
                self.stage_admission_denial(WorthQueryWorkflowAdvanceDenialKind::RuntimeAuthority(
                    denial.kind(),
                ))
            })?;
        Ok(WorthQueryWorkflowStageRuntimeAdmission { _private: () })
    }

    fn pending_stage(
        &mut self,
        stage_identity: &str,
    ) -> Result<
        worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        WorthQueryWorkflowAdvanceDenial,
    > {
        self.counters.stage_admission_checks += 1;
        self.counters.stage_index_lookups += 1;
        let stage = self.graph.stage(stage_identity).cloned().ok_or_else(|| {
            self.stage_admission_denial(WorthQueryWorkflowAdvanceDenialKind::UnknownStage)
        })?;
        if self.completed.contains(stage_identity) {
            return Err(self.stage_admission_denial(
                WorthQueryWorkflowAdvanceDenialKind::StageAlreadyCompleted,
            ));
        }
        Ok(stage)
    }

    fn validate_stage_predecessors(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        for predecessor in stage.predecessors() {
            self.counters.predecessor_checks += 1;
            if !self.completed.contains(predecessor) {
                return Err(self.stage_admission_denial(
                    WorthQueryWorkflowAdvanceDenialKind::PredecessorIncomplete(predecessor.clone()),
                ));
            }
        }
        Ok(())
    }

    fn validate_stage_capabilities(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        for required in stage.required_capabilities() {
            self.counters.required_capability_checks += 1;
            if !self
                .bound
                .operation()
                .domain_authority()
                .required_capabilities()
                .iter()
                .any(|installed| installed.satisfies_operation_requirement(*required))
            {
                return Err(self.stage_admission_denial(
                    WorthQueryWorkflowAdvanceDenialKind::RequiredCapability(
                        required.as_str().into(),
                    ),
                ));
            }
        }
        Ok(())
    }

    fn validate_stage_domains(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        for role in &stage.semantics().required_domain_roles {
            self.counters.required_domain_checks += 1;
            if !self
                .bound
                .required_domain_roles()
                .any(|bound| bound == role.as_str())
            {
                return Err(self.stage_admission_denial(
                    WorthQueryWorkflowAdvanceDenialKind::RequiredDomain(role.as_str().into()),
                ));
            }
        }
        Ok(())
    }

    fn stage_admission_denial(
        &self,
        kind: WorthQueryWorkflowAdvanceDenialKind,
    ) -> WorthQueryWorkflowAdvanceDenial {
        WorthQueryWorkflowAdvanceDenial::new(kind, self.counters)
    }
}
