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
    /// Narrow installed native source qualification. Other original owners keep
    /// the whole-family gate; installed meaning grants no current request authority.
    pub fn admit_native_correction_source_owner(
        &self,
        contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<(), Denial> {
        if !self.is_directory_home() {
            return Ok(());
        }
        match self.admit_directory_original_shape(contracts, correction)? {
            Admission::NativeRecordedCorrection => Ok(()),
            Admission::ExistingShape | Admission::ExternalOriginalRecoveryOnly => {
                self.admit_retained_original_operation_owner()
            }
        }
    }

    /// Preflight of the installed inverse for a genuine correction handoff.
    /// The publication owner separately requires private Prepared/Replay provenance.
    pub fn admit_native_correction_inverse(
        &self,
        contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
    ) -> Result<(), Denial> {
        if !self.is_directory_home() {
            return Ok(());
        }
        use worth_query_installation::facade::{
            InstalledExternalEffectContract, PublishedAftermathPosture,
        };
        if contracts.body().is_some()
            || !matches!(
                contracts.external_effect(),
                InstalledExternalEffectContract::None
            )
            || contracts.aftermath().is_some_and(|aftermath| {
                aftermath.published_posture() != PublishedAftermathPosture::Irreversible
                    || aftermath.mechanism().is_some()
            })
        {
            return Err(Denial::UnsupportedCorrectionCompensation);
        }
        Ok(())
    }

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
                if let Some(binding) = correction {
                    return compensation::classify_native(
                        aftermath.expect("compensation belongs to this aftermath"),
                        binding,
                        usage,
                    );
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
