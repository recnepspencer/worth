//! Durable correction shapes are admitted from installed original meaning.
use super::{
    DirectoryCorrectionShapeAdmission as Admission, DirectoryCorrectionShapeUse as Use,
    WorthQueryDurableStateAdmissionDenial as Denial, WorthQueryProductRuntime,
};
mod compensation;
use worth_query_installation::facade::{
    InstalledCorrectionBinding, InstalledCorrectionMechanism, WorthQueryInstalledAftermathContract,
};
impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn admit_directory_original_shape(
        &self,
        contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<Admission, Denial> {
        self.classify_directory_correction_shape(
            contracts.aftermath(),
            correction,
            Use::OriginalExecution(contracts),
        )
    }

    pub(in crate::domain_computation) fn admit_directory_correction_shape(
        &self,
        aftermath: Option<&WorthQueryInstalledAftermathContract>,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<(), Denial> {
        self.classify_directory_correction_shape(aftermath, correction, Use::CorrectionPreparation)
            .map(|_requirements| ())
    }

    fn classify_directory_correction_shape(
        &self,
        aftermath: Option<&WorthQueryInstalledAftermathContract>,
        correction: Option<&InstalledCorrectionBinding>,
        usage: Use<'_>,
    ) -> Result<Admission, Denial> {
        if !self.is_directory_home() {
            return Ok(Admission::ExistingShape);
        }
        let created_entity_material = match aftermath.and_then(|contract| contract.mechanism()) {
            Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) => {
                if !inverse.preimage_demand().relations().is_empty() {
                    return Err(Denial::UnsupportedCorrectionRelationPreimage);
                }
                inverse.preimage_demand().is_empty()
            }
            Some(InstalledCorrectionMechanism::Compensation(_)) => {
                if correction.is_some() {
                    return Err(Denial::UnsupportedCorrectionCompensation);
                }
                return compensation::classify(
                    aftermath.expect("compensation belongs to this aftermath"),
                    usage,
                );
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
        Ok(Admission::ExistingShape)
    }
}
