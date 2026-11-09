use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryExistingTruthProbeRequest;

use super::handoff_execution_binding_identity;
use crate::intent_admission::{
    WorthQueryExistingTruthProbeExecutionHandoff, WorthQueryIntentAdmissionCoveredEntrypoint,
    WorthQueryIntentAdmissionExecutionSeam, WorthQueryIntentAdmissionFamily,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryExistingTruthProbeExecutionBinding {
    handoff: WorthQueryExistingTruthProbeExecutionHandoff,
    binding_digest: String,
}

impl WorthQueryExistingTruthProbeExecutionBinding {
    pub(crate) fn from_handoff(
        handoff: WorthQueryExistingTruthProbeExecutionHandoff,
    ) -> Result<Self, IdentityDerivationDenial> {
        let binding_digest = handoff_execution_binding_identity(
            "existing-truth-probe-execution",
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

    pub fn request(&self) -> &WorthQueryExistingTruthProbeRequest {
        self.handoff.request()
    }

    pub fn handoff(&self) -> &WorthQueryExistingTruthProbeExecutionHandoff {
        &self.handoff
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }
}
