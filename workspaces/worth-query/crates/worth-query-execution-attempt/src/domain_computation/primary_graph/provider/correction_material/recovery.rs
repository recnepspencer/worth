//! The actual material receiver services funded recovery use handoffs.
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositePublicationAttemptIdentity,
};
impl WorthQueryPrimaryGraphProvider {
    pub fn acquire_recovery_material(
        &self,
        original: DurableCompositePublicationAttemptIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CorrectionMaterialPinIdentity, Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .acquire_recovery_pin(original, request)
    }
    pub fn release_recovery_material(
        &self,
        identity: CorrectionMaterialPinIdentity,
        original: DurableCompositePublicationAttemptIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .release_recovery_pin(identity, original, request)
    }
    pub fn settled_recovery_material_pins(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        Vec<(
            CorrectionMaterialPinIdentity,
            DurableCompositePublicationAttemptIdentity,
        )>,
        Denial,
    > {
        self.correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?
            .settled_recovery_pins(request)
    }
}
