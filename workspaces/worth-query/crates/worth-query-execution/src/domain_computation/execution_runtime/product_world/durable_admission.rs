//! Directory capability admission uses Host's one exhaustive support policy.
mod correction_shape;
use super::WorthQueryProductRuntime;
use worth_foundational::facade::DurableStateFamily;

/// Installed-use distinction; neither variant carries execution authority.
pub(in crate::domain_computation) enum DirectoryCorrectionShapeUse<'a> {
    OriginalExecution(
        &'a worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
    ),
    CorrectionPreparation,
}

/// Requirements to finish at the actual operation admission owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum DirectoryCorrectionShapeAdmission {
    ExistingShape,
    ExternalOriginalRecoveryOnly,
    NativeRecordedCorrection,
}

impl DirectoryCorrectionShapeAdmission {
    pub(in crate::domain_computation) fn requires_external_original_custody(self) -> bool {
        match self {
            Self::ExistingShape | Self::NativeRecordedCorrection => false,
            Self::ExternalOriginalRecoveryOnly => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDurableStateAdmissionDenial {
    PolicyUnavailable,
    UnsupportedCorrectionRelationPreimage,
    UnsupportedCorrectionCoupledOutputs,
    UnsupportedCorrectionCompensation,
    UnsupportedCorrectionOutputMaterial,
    UnsupportedCorrectionReapplication,
    UnsupportedDivergentCorrection,
    UndeclaredSupplierCapabilities,
    Unsupported {
        family: DurableStateFamily,
        supported_from_phase: u8,
    },
}
impl std::fmt::Display for WorthQueryDurableStateAdmissionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "directory capability admission refused: {self:?}")
    }
}
impl std::error::Error for WorthQueryDurableStateAdmissionDenial {}

impl WorthQueryProductRuntime {
    /// Admit retention of an original operation under this actual Home owner.
    /// Removed with the graceful original-owner readmission cutover in H9.
    pub fn admit_retained_original_operation_owner(
        &self,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        self.admit_directory_capability(
            crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability::RetainedOriginalOperationOwner,
        )
    }

    pub(in crate::domain_computation) fn admit_directory_family(
        &self,
        family: DurableStateFamily,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        if self.owner.owner_identity().durable().is_none() {
            return Ok(());
        }
        self.admit_directory_requirement(family, 0)
    }

    fn admit_directory_requirement(
        &self,
        family: DurableStateFamily,
        required_phase: u8,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        if !self.is_directory_home() {
            return Ok(());
        }
        let support = self
            .directory_support
            .ok_or(WorthQueryDurableStateAdmissionDenial::PolicyUnavailable)?;
        match support(family, required_phase) {
            None => Ok(()),
            Some(supported_from_phase) => Err(WorthQueryDurableStateAdmissionDenial::Unsupported {
                family,
                supported_from_phase,
            }),
        }
    }
    pub(in crate::domain_computation) fn admit_directory_capability(
        &self,
        capability: crate::domain_computation::primary_graph::WorthQueryOperationDurableCapability,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        let (families, phase) = capability.requirements();
        for &family in families {
            self.admit_directory_requirement(family, phase)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation) fn is_directory_home(&self) -> bool {
        self.owner.owner_identity().durable().is_some()
    }
}

mod court_doors_for_test;
