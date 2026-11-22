//! Restored acceptance keeps the actual owner's distinct provenance.
use super::super::WorthQueryAcceptedOutputCheckpointIdentity;
use crate::domain_computation::primary_graph::output_lineage::HistoricalAcceptedOrigin;

#[derive(Clone)]
pub enum RestoredOutputOrigin {
    Checkpoint(WorthQueryAcceptedOutputCheckpointIdentity),
    HistoricalSelection(HistoricalAcceptedOrigin),
}
impl RestoredOutputOrigin {
    pub fn key(&self) -> [u8; 32] {
        match self {
            Self::Checkpoint(identity) => identity.idempotency_key,
            Self::HistoricalSelection(origin) => origin.key(),
        }
    }
    pub fn partition(&self) -> [u8; 32] {
        match self {
            Self::Checkpoint(identity) => identity.source_partition,
            Self::HistoricalSelection(origin) => {
                *origin.publication().meaning().source_partition.bytes()
            }
        }
    }
    pub fn same_owner(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Checkpoint(left), Self::Checkpoint(right)) => left == right,
            (Self::HistoricalSelection(left), Self::HistoricalSelection(right)) => {
                left.same_owner(right)
            }
            _ => false,
        }
    }
}
