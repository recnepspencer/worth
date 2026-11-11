use super::{
    WorthQueryReadCompositionPhaseGate, WorthQueryReadCompositionPhaseOneCloseout,
    WorthQueryReadCompositionSupportReport, WorthQueryWorkspace,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl WorthQueryWorkspace {
    pub fn public_read_composition_support_report(&self) -> WorthQueryReadCompositionSupportReport {
        self.runtime.public_read_composition_support_report()
    }

    pub fn public_read_composition_phase_one_closeout(
        &self,
    ) -> Result<WorthQueryReadCompositionPhaseOneCloseout, IdentityDerivationDenial> {
        self.runtime.public_read_composition_phase_one_closeout()
    }

    pub fn public_read_composition_phase_gate(
        &self,
    ) -> Result<WorthQueryReadCompositionPhaseGate, IdentityDerivationDenial> {
        self.runtime.public_read_composition_phase_gate()
    }
}
