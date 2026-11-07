use super::handoff_execution_binding_identity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    WorthQueryDerivedInspectionExecutionHandoff, WorthQueryDerivedMaterializationExecutionHandoff,
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionExecutionSeam,
    WorthQueryIntentAdmissionFamily,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDerivedMaterializationExecutionBinding {
    handoff: WorthQueryDerivedMaterializationExecutionHandoff,
    binding_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryDerivedInspectionExecutionBinding {
    handoff: WorthQueryDerivedInspectionExecutionHandoff,
    binding_digest: String,
}

impl WorthQueryDerivedMaterializationExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryDerivedMaterializationExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "derived-materialization-execution",
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

    pub fn view_name(&self) -> &str {
        self.handoff.view_name()
    }

    pub fn handoff(&self) -> &WorthQueryDerivedMaterializationExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}

impl WorthQueryDerivedInspectionExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryDerivedInspectionExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "derived-inspection-execution",
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

    pub fn view_name(&self) -> &str {
        self.handoff.view_name()
    }

    pub fn handoff(&self) -> &WorthQueryDerivedInspectionExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}
