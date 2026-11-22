use crate::domain_computation::provider_session::graph_provider::{
    WorthQueryBoundGraphExecutionAssociation, WorthQueryCompletedDomainEvidenceDerivation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConvergenceDomainEvidenceBindingDenial {
    StaleInstallationGeneration,
    DirectOperationRequired,
    WorkflowOperationRequired,
    StageNotInstalled,
    ArtifactContractRequired,
    ReceiptAssociationRequired,
    ExecutionAssociationMismatch,
    EmptyRunIdentity,
    EmptyCandidateSelectionKey,
}

pub struct WorthQueryConvergenceDomainEvidenceBinding {
    derivation: WorthQueryCompletedDomainEvidenceDerivation,
}

impl WorthQueryConvergenceDomainEvidenceBinding {
    pub fn from_completed_execution(
        derivation: WorthQueryCompletedDomainEvidenceDerivation,
    ) -> Self {
        Self { derivation }
    }

    pub fn contract(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>
    {
        Some(self.derivation.contract())
    }

    pub fn candidate_selection_key(&self) -> &str {
        self.derivation.candidate_selection_key()
    }

    pub fn candidate_occurrence_identity(&self) -> &str {
        self.derivation.candidate_occurrence_identity()
    }

    pub fn belongs_to_execution(
        &self,
        association: &WorthQueryBoundGraphExecutionAssociation,
    ) -> bool {
        self.derivation.execution() == association
    }
}
