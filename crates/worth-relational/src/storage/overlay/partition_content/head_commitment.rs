use super::*;
use crate::storage::substrate::HistoryAppendFrontier;

/// History Merkle right spines carry enough proof to check current content
/// against the original region leaf and to append without reading old values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct PartitionHistoryFrontiers {
    pub(super) entities: Vec<(u64, HistoryAppendFrontier)>,
    pub(super) relations: Vec<(u64, HistoryAppendFrontier)>,
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
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: &PartitionState,
    ) -> Result<PartitionHistoryFrontiers, PartitionContentDigestError> {
        lane.recover(execution, |lane, work| {
            Ok(PartitionHistoryFrontiers {
                entities: self
                    .entities
                    .history_frontiers(lane, &partition.entity_arena, work)?,
                relations: self.relations.history_frontiers(
                    lane,
                    &partition.relation_arena,
                    work,
                )?,
            })
        })
    }

    /// The caller must compare the resulting digest with the committed leaf.
    /// Frontier structure alone never promotes supplied hashes into authority.
    pub(crate) fn capture_head(
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition: &PartitionState,
        symbols: &dyn crate::symbols::data::SymbolTextLookup,
        frontiers: &PartitionHistoryFrontiers,
    ) -> Result<Self, PartitionContentDigestError> {
        lane.recover(execution, |lane, work| {
            let seed = Self {
                entities: records::RecordContentIndex::from_history_frontiers(
                    lane,
                    work,
                    Some(execution),
                    &frontiers.entities,
                    &partition.entity_arena,
                )?,
                relations: records::RecordContentIndex::from_history_frontiers(
                    lane,
                    work,
                    Some(execution),
                    &frontiers.relations,
                    &partition.relation_arena,
                )?,
                ..Self::default()
            };
            Self::capture_seeded(lane, work, Some(execution), partition, symbols, seed)
                .map(|(commitment, _)| commitment)
        })
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
