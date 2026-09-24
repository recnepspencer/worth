use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use dashmap::DashMap;

use crate::identity::data::KindId;
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::performance::data::CandidateInputCounts;
use crate::validation::engine::state_view::{VisibleEntityMetadata, VisibleRelationMetadata};
use crate::validation::engine::{
    InvariantExecutionRequest, InvariantObservation, InvariantRuntimeView,
};

/// A request-local observation basis. No entry can outlive its engine execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CandidateInputBasis {
    Enforcement,
    BeforeImage,
    Committed,
}

pub(crate) struct SharedCandidateInputs {
    pub(super) entries: CandidateInputEntries,
    pub(super) sharing_enabled: bool,
}

impl CandidateInputBasis {
    pub(crate) fn enforcement(observation: &InvariantObservation<'_>) -> Self {
        if observation.enforcement_uses_committed_state() {
            Self::Committed
        } else {
            Self::Enforcement
        }
    }

    pub(crate) fn before_image(observation: &InvariantObservation<'_>) -> Self {
        if observation.before_image_uses_committed_state() {
            Self::Committed
        } else {
            Self::BeforeImage
        }
    }
}

#[derive(Default)]
pub(super) struct CandidateInputEntries {
    pub(super) entities: DashMap<
        (CandidateInputBasis, VersionId, EntityId),
        Arc<OnceLock<Option<VisibleEntityMetadata>>>,
    >,
    pub(super) relations: DashMap<
        (CandidateInputBasis, VersionId, RelationId),
        Arc<OnceLock<Option<VisibleRelationMetadata>>>,
    >,
    pub(super) entity_aspects:
        DashMap<(CandidateInputBasis, VersionId, EntityId), Arc<OnceLock<Option<usize>>>>,
    pub(super) relation_aspects:
        DashMap<(CandidateInputBasis, VersionId, RelationId), Arc<OnceLock<Option<usize>>>>,
    pub(super) adjacency:
        DashMap<(CandidateInputBasis, VersionId, EntityId, bool), Arc<OnceLock<Arc<[RelationId]>>>>,
    pub(super) adjacency_counts:
        DashMap<(CandidateInputBasis, VersionId, EntityId, bool), Arc<OnceLock<usize>>>,
    pub(super) touched_entities:
        DashMap<(CandidateInputBasis, VersionId), Arc<OnceLock<Vec<EntityId>>>>,
    pub(super) touched_relations:
        DashMap<(CandidateInputBasis, VersionId), Arc<OnceLock<Vec<RelationId>>>>,
    pub(super) touched_partitions:
        DashMap<(CandidateInputBasis, VersionId), Arc<OnceLock<Option<Arc<[PartitionId]>>>>>,
    pub(super) touched_entity_slots:
        DashMap<(CandidateInputBasis, VersionId, PartitionId), Arc<OnceLock<Option<Arc<[usize]>>>>>,
    pub(super) touched_relation_slots:
        DashMap<(CandidateInputBasis, VersionId, PartitionId), Arc<OnceLock<Option<Arc<[usize]>>>>>,
    pub(super) entity_reads: AtomicUsize,
    pub(super) relation_reads: AtomicUsize,
    pub(super) entity_aspect_reads: AtomicUsize,
    pub(super) relation_aspect_reads: AtomicUsize,
    pub(super) adjacency_gathers: AtomicUsize,
    pub(super) adjacency_count_reads: AtomicUsize,
    pub(super) adjacency_relation_ids: AtomicUsize,
    pub(super) touched_entity_gathers: AtomicUsize,
    pub(super) touched_relation_gathers: AtomicUsize,
    pub(super) touched_partition_gathers: AtomicUsize,
    pub(super) touched_entity_slot_gathers: AtomicUsize,
    pub(super) touched_relation_slot_gathers: AtomicUsize,
    pub(super) reuse_hits: AtomicUsize,
}

impl SharedCandidateInputs {
    #[cfg(test)]
    pub(crate) fn sharing_for_test() -> Self {
        Self {
            entries: CandidateInputEntries::default(),
            sharing_enabled: true,
        }
    }

    /// Installation declares the overlap. Runtime reads only fill exact entries
    /// demanded by the participating rules on this observation.
    pub(crate) fn from_installed(
        runtime: &InvariantRuntimeView<'_>,
        request: &InvariantExecutionRequest<'_>,
    ) -> Option<Self> {
        let mut entity_kinds = BTreeSet::<KindId>::new();
        let mut relation_kinds = BTreeSet::<KindId>::new();
        let mut rule_count = 0;
        let mut sharing_enabled = false;
        for registration in runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .filter(|registration| request.includes_custom_registration(registration))
        {
            rule_count += 1;
            let access = registration.access_contract();
            let rule_entities = access
                .read_entity_kinds
                .iter()
                .chain(&access.affected_entity_kinds);
            let rule_relations = access
                .read_relation_kinds
                .iter()
                .chain(&access.affected_relation_kinds);
            if rule_entities
                .clone()
                .any(|kind| entity_kinds.contains(kind))
                || rule_relations
                    .clone()
                    .any(|kind| relation_kinds.contains(kind))
            {
                sharing_enabled = true;
            }
            entity_kinds.extend(rule_entities.copied());
            relation_kinds.extend(rule_relations.copied());
        }
        (rule_count > 0).then(|| Self {
            entries: CandidateInputEntries::default(),
            sharing_enabled,
        })
    }

    pub(crate) fn counters(&self) -> CandidateInputCounts {
        let entries = &self.entries;
        CandidateInputCounts {
            entity_reads: entries.entity_reads.load(Ordering::Relaxed),
            relation_reads: entries.relation_reads.load(Ordering::Relaxed),
            entity_aspect_reads: entries.entity_aspect_reads.load(Ordering::Relaxed),
            relation_aspect_reads: entries.relation_aspect_reads.load(Ordering::Relaxed),
            adjacency_gathers: entries.adjacency_gathers.load(Ordering::Relaxed),
            adjacency_count_reads: entries.adjacency_count_reads.load(Ordering::Relaxed),
            adjacency_relation_ids: entries.adjacency_relation_ids.load(Ordering::Relaxed),
            touched_entity_gathers: entries.touched_entity_gathers.load(Ordering::Relaxed),
            touched_relation_gathers: entries.touched_relation_gathers.load(Ordering::Relaxed),
            touched_partition_gathers: entries.touched_partition_gathers.load(Ordering::Relaxed),
            touched_entity_slot_gathers: entries
                .touched_entity_slot_gathers
                .load(Ordering::Relaxed),
            touched_relation_slot_gathers: entries
                .touched_relation_slot_gathers
                .load(Ordering::Relaxed),
            reuse_hits: entries.reuse_hits.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::{KindId, PartitionId};

    #[test]
    fn absence_and_foreign_or_changed_basis_do_not_alias() {
        let inputs = SharedCandidateInputs {
            entries: CandidateInputEntries::default(),
            sharing_enabled: true,
        };
        let id = EntityId::new(PartitionId::main(), 7, 1);
        let value = VisibleEntityMetadata {
            entity_id: id,
            kind_id: KindId(2),
        };
        let mut reads = 0;
        assert!(inputs
            .entity(CandidateInputBasis::Enforcement, VersionId(1), id, || {
                reads += 1;
                None
            })
            .is_none());
        assert!(inputs
            .entity(CandidateInputBasis::Enforcement, VersionId(1), id, || {
                reads += 1;
                Some(value.clone())
            })
            .is_none());
        assert_eq!(reads, 1);
        assert_eq!(
            inputs
                .entity(CandidateInputBasis::Enforcement, VersionId(2), id, || {
                    reads += 1;
                    Some(value.clone())
                })
                .map(|metadata| metadata.kind_id),
            Some(KindId(2))
        );
        assert!(inputs
            .entity(CandidateInputBasis::BeforeImage, VersionId(1), id, || {
                reads += 1;
                None
            })
            .is_none());
        assert_eq!(reads, 3);
        assert_eq!(inputs.counters().entity_reads, 3);

        assert!(inputs
            .entity_aspect(CandidateInputBasis::Enforcement, VersionId(1), id, || None)
            .is_none());
        assert!(inputs
            .entity_aspect(
                CandidateInputBasis::Enforcement,
                VersionId(1),
                id,
                || panic!("absent field state was reread")
            )
            .is_none());
        assert!(inputs
            .entity_aspect(CandidateInputBasis::Enforcement, VersionId(2), id, || None)
            .is_none());
        assert!(inputs
            .entity_aspect(CandidateInputBasis::BeforeImage, VersionId(1), id, || None)
            .is_none());
        assert_eq!(inputs.counters().entity_aspect_reads, 3);
    }
}
