//! Real pre-effect storage for a detached checkpoint and its independent fold.
mod preparation;
mod storage;
#[cfg(test)]
mod tests;
pub(in crate::physical_runtime) use storage::{
    CheckpointCertificateViews, FundedCheckpointBufferPreparation,
    FundedCheckpointCommandBufferLease, FundedCheckpointFrame,
};
pub(in crate::physical_runtime::durability::publication::current_root_owner) use storage::{
    CheckpointPreparation, ReusableCheckpointSlot, SealedCheckpointStorage,
};

use super::super::certificate_capacity::CheckpointCustodyDenial;
use super::ReleaseCertificateCapacityDenial;

impl From<ReleaseCertificateCapacityDenial> for CheckpointCustodyDenial {
    fn from(value: ReleaseCertificateCapacityDenial) -> Self {
        match value {
            ReleaseCertificateCapacityDenial::Resident(cause) => Self::Backing(cause),
            _ => Self::ReleaseCertificateUnavailable,
        }
    }
}
