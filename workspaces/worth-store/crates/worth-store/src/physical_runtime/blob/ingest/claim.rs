use crate::physical_runtime::{
    durability::PhysicalBlobSessionClaimDenial, PhysicalReadProtectionDenial,
};

/// Live, Store-local attempt admission. This is not a durable session fate;
/// selected C5 records remain authoritative after a crash or reopen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobIngestClaimDenial {
    CompetingSession,
    Capacity,
    MetadataUnavailable,
    ReadProtection(PhysicalReadProtectionDenial),
    CheckpointChanged,
    ClaimLost,
    ReclaimFenced,
}

impl From<PhysicalBlobSessionClaimDenial> for BlobIngestClaimDenial {
    fn from(denial: PhysicalBlobSessionClaimDenial) -> Self {
        match denial {
            PhysicalBlobSessionClaimDenial::CompetingSession => Self::CompetingSession,
            PhysicalBlobSessionClaimDenial::Capacity => Self::Capacity,
            PhysicalBlobSessionClaimDenial::MetadataUnavailable => Self::MetadataUnavailable,
            PhysicalBlobSessionClaimDenial::Protection(cause) => Self::ReadProtection(cause),
            PhysicalBlobSessionClaimDenial::CheckpointChanged => Self::CheckpointChanged,
            PhysicalBlobSessionClaimDenial::ClaimLost => Self::ClaimLost,
            PhysicalBlobSessionClaimDenial::ReclaimFenced => Self::ReclaimFenced,
        }
    }
}
