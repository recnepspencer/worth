//! Restored acceptance keeps the actual owner's distinct provenance.
use super::super::WorthQueryAcceptedOutputCheckpointIdentity;

#[derive(Clone)]
pub enum RestoredOutputOrigin {
    Checkpoint(WorthQueryAcceptedOutputCheckpointIdentity),
}
impl RestoredOutputOrigin {
    pub fn key(&self) -> [u8; 32] {
        match self {
            Self::Checkpoint(identity) => identity.idempotency_key
        }
    }
    pub fn partition(&self) -> [u8; 32] {
        match self {
            Self::Checkpoint(identity) => identity.source_partition
        }
    }
    pub fn same_owner(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Checkpoint(left), Self::Checkpoint(right)) => left == right
        }
    }
}
