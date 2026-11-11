//! Protected original membership readmitted under the actual consuming owner.
use super::{SignalBranchAdvanceArtifact, SignalRecordedSourcePoint};
use crate::branch::AdmittedSignalBranchBasis;

/// Recovered evidence is distinct from a live committed result. It contains no
/// derived TransactionResult and grants no claim that this point is current.
#[derive(Debug)]
pub struct SignalReadmittedSourcePoint {
    point: SignalRecordedSourcePoint,
    original: SignalBranchAdvanceArtifact,
    basis: AdmittedSignalBranchBasis,
}
impl SignalReadmittedSourcePoint {
    pub(crate) fn from_owner(
        point: SignalRecordedSourcePoint,
        original: SignalBranchAdvanceArtifact,
        basis: AdmittedSignalBranchBasis,
    ) -> Self {
        Self {
            point,
            original,
            basis,
        }
    }
    pub fn source_point(&self) -> &SignalRecordedSourcePoint {
        &self.point
    }
    pub fn original(&self) -> &SignalBranchAdvanceArtifact {
        &self.original
    }
    pub fn basis(&self) -> &AdmittedSignalBranchBasis {
        &self.basis
    }
}

#[derive(Debug)]
pub enum SignalSourcePointReadmissionDenial {
    Original(super::SignalOwnerReadmissionDenial),
    Source(crate::branch::owner_services::SignalDurableSourceDenial),
    Image(super::SignalAuthoredImageDenial),
    RetiredOriginal,
}
