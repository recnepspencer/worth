//! Authenticated history facts that follow one record's omitted prefix.
use super::HistoryAppendFrontier;
use crate::storage::partition::RegionRelationProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordHistoryOpening {
    pub frontier: HistoryAppendFrontier,
    pub kind_commitment: [u8; 32],
    pub historical_relations: Vec<RegionRelationProjection>,
}

impl RecordHistoryOpening {
    pub fn owned_allocation_bytes(&self) -> u64 {
        self.frontier.owned_allocation_bytes()
            + (self.historical_relations.capacity()
                * std::mem::size_of::<RegionRelationProjection>()) as u64
            + self
                .historical_relations
                .iter()
                .map(|projection| {
                    ((projection.outgoing.len() + projection.incoming.len())
                        * std::mem::size_of::<usize>()) as u64
                })
                .sum::<u64>()
    }
}
