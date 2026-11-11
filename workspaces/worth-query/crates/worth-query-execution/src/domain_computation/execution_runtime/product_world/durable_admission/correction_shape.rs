//! Durable correction shapes are admitted from installed original meaning.
use super::{WorthQueryDurableStateAdmissionDenial as Denial, WorthQueryProductRuntime};
use worth_query_installation::facade::{
    InstalledCorrectionBinding, InstalledCorrectionMechanism, WorthQueryInstalledAftermathContract,
};
impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn admit_directory_correction_shape(
        &self,
        aftermath: Option<&WorthQueryInstalledAftermathContract>,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<(), Denial> {
        if !self.is_directory_home() {
            return Ok(());
        }
        let created_entity_material = match aftermath.and_then(|contract| contract.mechanism()) {
            Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) => {
                if !inverse.preimage_demand().relations().is_empty() {
                    return Err(Denial::UnsupportedCorrectionRelationPreimage);
                }
                inverse.preimage_demand().is_empty()
            }
            Some(InstalledCorrectionMechanism::Compensation(_)) => {
                return Err(Denial::UnsupportedCorrectionCompensation);
            }
            None => false,
        };
        if correction.is_some_and(|binding| !binding.declaration().coupled_outputs().is_empty()) {
            return Err(Denial::UnsupportedCorrectionCoupledOutputs);
        }
        if created_entity_material
            || correction.is_some_and(|binding| {
                binding.declaration().authored_identity().is_some()
                    || !binding.declaration().source_output_fields().is_empty()
            })
        {
            return Err(Denial::UnsupportedCorrectionOutputMaterial);
        }
        Ok(())
    }
}
