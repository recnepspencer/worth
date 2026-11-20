//! Directory correction restrictions are checked against installed meaning.
mod correction_shape;
use super::WorthQueryProductRuntime;

/// Installed-use distinction; neither variant carries execution authority.
pub enum DirectoryCorrectionShapeUse<'a> {
    OriginalExecution(
        &'a worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
    ),
    CorrectionPreparation,
}

/// Requirements to finish at the actual operation admission owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryCorrectionShapeAdmission {
    ExistingShape,
    /// `inbound` records whether the installed external effect declares a
    /// canonical inbound completion; an original may settle by dispatch alone.
    ExternalOriginalRecoveryOnly {
        inbound: bool,
    },
    NativeRecordedCorrection,
}

impl DirectoryCorrectionShapeAdmission {
    /// `Some(inbound)` when the operation admission owner must still require
    /// external original custody from the supplier.
    pub fn external_original_inbound(self) -> Option<bool> {
        match self {
            Self::ExistingShape | Self::NativeRecordedCorrection => None,
            Self::ExternalOriginalRecoveryOnly { inbound } => Some(inbound),
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
    pub fn is_directory_home(&self) -> bool {
        self.owner.owner_identity().durable().is_some()
    }
}
