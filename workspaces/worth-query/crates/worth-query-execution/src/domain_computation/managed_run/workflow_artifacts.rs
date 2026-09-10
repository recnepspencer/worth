use std::sync::Arc;

use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactAccessAuthority, WorthQueryArtifactDenial,
    WorthQueryArtifactProductionAuthority, WorthQueryArtifactTransferAdmission,
    WorthQueryWorkflowArtifactAuthority,
};

pub struct WorthQueryManagedWorkflowArtifactAuthority<'run> {
    authority: &'run WorthQueryWorkflowArtifactAuthority,
}

impl<'run> WorthQueryManagedWorkflowArtifactAuthority<'run> {
    pub(super) fn new(authority: &'run WorthQueryWorkflowArtifactAuthority) -> Self {
        Self { authority }
    }

    pub fn run_identity(&self) -> &str {
        self.authority.run_identity()
    }

    pub fn registry(
        &self,
    ) -> Arc<crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactRegistry> {
        self.authority.registry()
    }

    pub fn production_authority(
        &self,
        stage_identity: &str,
    ) -> Result<Option<Arc<WorthQueryArtifactProductionAuthority>>, WorthQueryArtifactDenial> {
        self.authority.production_authority(stage_identity)
    }

    pub fn access_authority(
        &self,
        stage_identity: &str,
    ) -> Result<Option<Arc<WorthQueryArtifactAccessAuthority>>, WorthQueryArtifactDenial> {
        self.authority.access_authority(stage_identity)
    }

    pub fn transfer_admission(
        &self,
        predecessor_stage: &str,
        consumer_stage: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.authority
            .transfer_admission(predecessor_stage, consumer_stage)
    }

    pub fn input_validation_admission(
        &self,
        stage_identity: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.authority.input_validation_admission(stage_identity)
    }

    pub fn output_validation_admission(
        &self,
        stage_identity: &str,
    ) -> Result<WorthQueryArtifactTransferAdmission, WorthQueryArtifactDenial> {
        self.authority.output_validation_admission(stage_identity)
    }
}
