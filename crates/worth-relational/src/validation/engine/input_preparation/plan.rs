use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::allocation::{arc_bytes, CandidateInputAllocation};
use super::cache_map::CandidateInputMap;

use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::performance::data::CandidateInputCounts;
use crate::validation::engine::state_view::slot_resolution::AspectLocation;
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
    pub(super) allocation: CandidateInputAllocation,
}

impl CandidateInputBasis {
    pub(crate) fn custom_enforcement(observation: &InvariantObservation<'_>) -> Self {
        if observation.custom_uses_suspension_source() {
            Self::before_image(observation)
        } else {
            Self::enforcement(observation)
        }
    }

    pub(crate) fn custom_committed(observation: &InvariantObservation<'_>) -> Self {
        if observation.custom_uses_suspension_source() {
            Self::before_image(observation)
        } else {
            Self::Committed
        }
    }

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
    pub(super) entities: CandidateInputMap<
        (CandidateInputBasis, VersionId, EntityId),
        Option<VisibleEntityMetadata>,
    >,
    pub(super) relations: CandidateInputMap<
        (CandidateInputBasis, VersionId, RelationId),
        Option<VisibleRelationMetadata>,
    >,
    pub(super) entity_aspects:
        CandidateInputMap<(CandidateInputBasis, VersionId, EntityId), Option<AspectLocation>>,
    pub(super) relation_aspects:
        CandidateInputMap<(CandidateInputBasis, VersionId, RelationId), Option<AspectLocation>>,
    pub(super) adjacency:
        CandidateInputMap<(CandidateInputBasis, VersionId, EntityId, bool), Arc<[RelationId]>>,
    pub(super) adjacency_counts:
        CandidateInputMap<(CandidateInputBasis, VersionId, EntityId, bool), usize>,
    pub(super) touched_entities:
        CandidateInputMap<(CandidateInputBasis, VersionId), Arc<[EntityId]>>,
    pub(super) touched_relations:
        CandidateInputMap<(CandidateInputBasis, VersionId), Arc<[RelationId]>>,
    pub(super) touched_partitions:
        CandidateInputMap<(CandidateInputBasis, VersionId), Option<Arc<[PartitionId]>>>,
    pub(super) touched_entity_slots:
        CandidateInputMap<(CandidateInputBasis, VersionId, PartitionId), Option<Arc<[usize]>>>,
    pub(super) touched_relation_slots:
        CandidateInputMap<(CandidateInputBasis, VersionId, PartitionId), Option<Arc<[usize]>>>,
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
        Self::new(true, None).unwrap()
    }

    pub(super) fn new(
        sharing_enabled: bool,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<Self, crate::execution::PacketBudgetDenial> {
        let allocation = CandidateInputAllocation::new(lease, arc_bytes::<Self>())?;
        Ok(Self {
            entries: CandidateInputEntries::default(),
            sharing_enabled,
            allocation,
        })
    }

    /// Installation declares the overlap. Runtime reads only fill exact entries
    /// demanded by the participating rules on this observation.
    pub(crate) fn from_installed(
        runtime: &InvariantRuntimeView<'_>,
        request: &InvariantExecutionRequest<'_>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<Option<Self>, crate::execution::PacketBudgetDenial> {
        let registrations = &runtime.schema_contract_runtime.custom_invariant_registries;
        if lease.is_some() {
            return if registrations
                .iter()
                .any(|registration| request.includes_custom_registration(registration))
            {
                Self::new(true, lease).map(Some)
            } else {
                Ok(None)
            };
        }
        let mut entities = BTreeSet::new();
        let mut relations = BTreeSet::new();
        let mut rule_count = 0;
        let mut sharing_enabled = false;
        for registration in registrations
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
            sharing_enabled |= rule_entities.clone().any(|kind| entities.contains(kind))
                || rule_relations.clone().any(|kind| relations.contains(kind));
            entities.extend(rule_entities.copied());
            relations.extend(rule_relations.copied());
        }
        if rule_count == 0 {
            return Ok(None);
        }
        // Admission precedes both the owner Arc and every cache entry.
        Self::new(sharing_enabled, lease).map(Some)
    }

    pub(crate) fn failure(&self) -> Option<crate::execution::PacketBudgetDenial> {
        self.allocation.failure()
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
            allocation: CandidateInputAllocation::new(None, 0).unwrap(),
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
