use super::handoff_execution_binding_identity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionExecutionSeam,
    WorthQueryIntentAdmissionFamily, WorthQueryUnifiedInspectionExecutionHandoff,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryUnifiedInspectionExecutionBinding {
    handoff: WorthQueryUnifiedInspectionExecutionHandoff,
    binding_digest: String,
}

impl WorthQueryUnifiedInspectionExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryUnifiedInspectionExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "unified-inspection-execution",
            handoff.handoff_digest(),
        )?;
        Ok(Self {
            handoff,
            binding_digest,
        })
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.handoff.family()
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.handoff.entrypoint()
    }

    pub fn execution_seam(&self) -> WorthQueryIntentAdmissionExecutionSeam {
        self.handoff.execution_seam()
    }

    pub fn seed(&self) -> &crate::intent_admission::WorthQueryGenericInspectionIntentSeed {
        self.handoff.seed()
    }

    pub fn handoff(&self) -> &WorthQueryUnifiedInspectionExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}
