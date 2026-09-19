use std::sync::Arc;

use super::{
    WorthQueryArtifactAccessAuthority, WorthQueryArtifactDenial, WorthQueryArtifactDenialKind,
    WorthQueryArtifactProductionAuthority, WorthQueryArtifactProductionAuthorityParts,
    WorthQueryArtifactProductionGeneration, WorthQueryArtifactProductionGenerationPending,
    WorthQueryArtifactTransferAdmission, WorthQueryArtifactTransferAdmissionParts,
    WorthQueryWorkflowArtifactRegistry,
};
use crate::domain_computation::operation_binding::WorthQueryExecutionBoundOperationAuthority;

pub struct WorthQueryWorkflowArtifactAuthority {
    run_identity: Arc<str>,
    binding: Arc<WorthQueryExecutionBoundOperationAuthority>,
    registry: Arc<WorthQueryWorkflowArtifactRegistry>,
}

impl WorthQueryWorkflowArtifactAuthority {
    pub(crate) fn mint(
        binding: Arc<WorthQueryExecutionBoundOperationAuthority>,
        provider_session_identity: &str,
        run_generation: u64,
    ) -> Result<Self, WorthQueryArtifactDenial> {
        if !binding.is_current_installation_generation() {
            return Err(denial(
                WorthQueryArtifactDenialKind::StaleInstallationGeneration,
                "workflow artifact authority requires the current installation generation",
            ));
        }
        let run_identity = crate::execution_digest::hash_parts(&[
            "worth_query_workflow_run_v3".into(),
            format!("binding:{}", binding.binding_identity()),
            format!("operation:{}", binding.operation_identity()),
            format!("basis:{}", binding.basis_identity()),
            format!("provider-session:{provider_session_identity}"),
            format!("run-generation:{run_generation}"),
        ]);
        let registry = Arc::new(WorthQueryWorkflowArtifactRegistry::new(
            run_identity.clone(),
        ));
        Ok(Self {
            run_identity: Arc::from(run_identity),
            binding,
            registry,
        })
    }

    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub fn registry(&self) -> Arc<WorthQueryWorkflowArtifactRegistry> {
        Arc::clone(&self.registry)
    }

    pub(in crate::domain_computation) fn belongs_to_binding(
        &self,
        binding: &WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        std::ptr::eq(self.binding.as_ref(), binding)
    }

    pub fn production_authority(
        &self,
        stage_identity: &str,
    ) -> Result<Option<Arc<WorthQueryArtifactProductionAuthority>>, WorthQueryArtifactDenial> {
        self.validate_current_stage(stage_identity)?;
        let generation = self.registry.current_production_generation()?;
        self.mint_production_authority(stage_identity, generation)
    }

    pub(crate) fn production_authority_for_readmission(
        &self,
        stage_identity: &str,
        pending: &WorthQueryArtifactProductionGenerationPending,
    ) -> Result<Option<Arc<WorthQueryArtifactProductionAuthority>>, WorthQueryArtifactDenial> {
        self.validate_current_stage(stage_identity)?;
        if !pending.belongs_to(&self.registry) {
            return Err(denial(
                WorthQueryArtifactDenialKind::RunMismatch,
                "artifact generation transition belongs to another workflow run",
            ));
        }
        self.mint_production_authority(stage_identity, pending.generation())
    }

    fn mint_production_authority(
        &self,
        stage_identity: &str,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> Result<Option<Arc<WorthQueryArtifactProductionAuthority>>, WorthQueryArtifactDenial> {
        let contract = self
            .binding
            .workflow_stage_artifact_contracts(stage_identity)
            .and_then(|contracts| contracts.output())
            .cloned();
        Ok(contract.map(|contract| {
            WorthQueryArtifactProductionAuthority::mint(
                WorthQueryArtifactProductionAuthorityParts {
                    contract,
                    domain_authority: self.binding.retain_installed_domain_authority(),
                    operation_identity: self.binding.operation_identity().to_owned(),
                    binding_identity: self.binding.binding_identity().to_owned(),
                    run_identity: self.run_identity().to_owned(),
                    stage_identity: stage_identity.to_owned(),
                    basis_identity: self.binding.basis_identity().to_owned(),
                    registry: Arc::clone(&self.registry),
                    production_generation,
                },
            )
        }))
    }

    pub fn access_authority(
        &self,
        stage_identity: &str,
    ) -> Result<Option<Arc<WorthQueryArtifactAccessAuthority>>, WorthQueryArtifactDenial> {
        self.validate_current_stage(stage_identity)?;
        let contract = self
            .binding
            .workflow_stage_artifact_contracts(stage_identity)
            .and_then(|contracts| contracts.input())
            .cloned();
        Ok(contract.map(|contract| {
            WorthQueryArtifactAccessAuthority::mint(
                contract,
                self.binding.retain_installed_domain_authority(),
                self.binding.operation_identity(),
                self.binding.binding_identity(),
                self.run_identity(),
                stage_identity,
                self.binding.basis_identity(),
            )
        }))
    }

    pub fn transfer_admission(
        &self,
        predecessor_stage: &str,
        consumer_stage: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.validate_current_stage(consumer_stage)?;
        if !self
            .binding
            .admits_workflow_edge(predecessor_stage, consumer_stage)
        {
            return Err(denial(
                WorthQueryArtifactDenialKind::StageMismatch,
                "artifact transfer requires an installed workflow edge",
            ));
        }
        let expected = self
            .binding
            .workflow_stage_artifact_contracts(consumer_stage)
            .and_then(|contracts| contracts.input())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryArtifactDenialKind::ArtifactContractNotInstalled,
                    "consumer stage has no installed artifact input contract",
                )
            })?;
        Ok(self.admission(expected, predecessor_stage, consumer_stage))
    }

    pub fn input_validation_admission(
        &self,
        stage_identity: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.validate_current_stage(stage_identity)?;
        let expected = self
            .binding
            .workflow_stage_artifact_contracts(stage_identity)
            .and_then(|contracts| contracts.input())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryArtifactDenialKind::ArtifactContractNotInstalled,
                    "stage has no installed artifact input contract",
                )
            })?;
        Ok(self.admission(expected, stage_identity, stage_identity))
    }

    pub fn output_validation_admission(
        &self,
        stage_identity: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.validate_current_stage(stage_identity)?;
        let expected = self
            .binding
            .workflow_stage_artifact_contracts(stage_identity)
            .and_then(|contracts| contracts.output())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryArtifactDenialKind::ArtifactContractNotInstalled,
                    "stage has no installed artifact output contract",
                )
            })?;
        Ok(self.admission(expected, stage_identity, stage_identity))
    }

    fn validate_current_stage(&self, stage_identity: &str) -> Result<(), WorthQueryArtifactDenial> {
        if !self.binding.is_current_installation_generation() {
            return Err(denial(
                WorthQueryArtifactDenialKind::StaleInstallationGeneration,
                "artifact authority belongs to a stale installation generation",
            ));
        }
        if self
            .binding
            .workflow_stage_artifact_contracts(stage_identity)
            .is_none()
        {
            return Err(denial(
                WorthQueryArtifactDenialKind::StageMismatch,
                "workflow stage is not installed for the bound operation",
            ));
        }
        Ok(())
    }

    fn admission(
        &self,
        expected_contract: Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
        predecessor_stage: &str,
        consumer_stage: &str,
    ) -> WorthQueryArtifactTransferAdmission {
        WorthQueryArtifactTransferAdmission::mint(WorthQueryArtifactTransferAdmissionParts {
            expected_contract,
            domain_authority: self.binding.retain_installed_domain_authority(),
            operation_identity: self.binding.operation_identity().to_owned(),
            binding_identity: self.binding.binding_identity().to_owned(),
            run_identity: self.run_identity().to_owned(),
            predecessor_stage: predecessor_stage.to_owned(),
            consumer_stage: consumer_stage.to_owned(),
            basis_identity: self.binding.basis_identity().to_owned(),
        })
    }
}

fn denial(kind: WorthQueryArtifactDenialKind, detail: &'static str) -> WorthQueryArtifactDenial {
    WorthQueryArtifactDenial::new(kind, None, detail)
}
