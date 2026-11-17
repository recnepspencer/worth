//! Directory correction restrictions are checked against installed meaning.
mod correction_shape;
use super::WorthQueryProductRuntime;

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
    UnsupportedCorrectionRelationPreimage,
    UnsupportedCorrectionCoupledOutputs,
    UnsupportedCorrectionCompensation,
    UnsupportedCorrectionOutputMaterial,
    UnsupportedCorrectionReapplication,
    UnsupportedDivergentCorrection,
    UndeclaredSupplierCapabilities,
}
impl std::fmt::Display for WorthQueryDurableStateAdmissionDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "directory capability admission refused: {self:?}")
    }
}
impl std::error::Error for WorthQueryDurableStateAdmissionDenial {}

impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn is_directory_home(&self) -> bool {
        self.owner.owner_identity().durable().is_some()
    }
}
