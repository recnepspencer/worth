use super::*;
use crate::storage::substrate::HistoryAppendFrontier;

/// History Merkle right spines carry enough proof to check current content
/// against the original region leaf and to append without reading old values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PartitionHistoryFrontiers {
    entities: Vec<(u64, HistoryAppendFrontier)>,
    relations: Vec<(u64, HistoryAppendFrontier)>,
}

impl PartitionHistoryFrontiers {
    pub(crate) fn into_arenas(
        self,
    ) -> (
        Vec<(u64, HistoryAppendFrontier)>,
        Vec<(u64, HistoryAppendFrontier)>,
    ) {
        (self.entities, self.relations)
    }
}

impl PartitionContentCommitment {
    pub(crate) fn history_frontiers(
        &self,
        partition: &PartitionState,
    ) -> Option<PartitionHistoryFrontiers> {
        Some(PartitionHistoryFrontiers {
            entities: self.entities.history_frontiers(&partition.entity_arena)?,
            relations: self
                .relations
                .history_frontiers(&partition.relation_arena)?,
        })
    }

    /// The caller must compare the resulting digest with the committed leaf.
    /// Frontier structure alone never promotes supplied hashes into authority.
    pub(crate) fn capture_head(
        partition: &PartitionState,
        symbols: &StringInterner,
        frontiers: &PartitionHistoryFrontiers,
    ) -> Option<Self> {
        let seed = Self {
            entities: records::RecordContentIndex::from_history_frontiers(
                &frontiers.entities,
                &partition.entity_arena,
            )?,
            relations: records::RecordContentIndex::from_history_frontiers(
                &frontiers.relations,
                &partition.relation_arena,
            )?,
            ..Self::default()
        };
        Self::capture_seeded(partition, symbols, None, seed)
            .ok()
            .map(|(commitment, _)| commitment)
    }
}

impl PartitionContentCommitment {
    pub(crate) fn borrowed_history_frontiers<'a>(
        &'a self,
        partition: &'a PartitionState,
    ) -> impl serde::Serialize + 'a {
        #[derive(serde::Serialize)]
        struct View<E: serde::Serialize, R: serde::Serialize> {
            entities: E,
            relations: R,
        }
        View {
            entities: self.entities.frontier_view(&partition.entity_arena),
            relations: self.relations.frontier_view(&partition.relation_arena),
        }
    }
}
