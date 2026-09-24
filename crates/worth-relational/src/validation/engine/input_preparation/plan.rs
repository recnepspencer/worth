use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use crate::identity::data::KindId;
use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::validation::engine::state_view::{VisibleEntityMetadata, VisibleRelationMetadata};
use crate::validation::engine::{InvariantExecutionRequest, InvariantRuntimeView};
use worth_foundational::facade::AuthoritativeRecordAspectState;

/// A request-local observation basis. No entry can outlive its engine execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CandidateInputBasis {
    Enforcement,
    BeforeImage,
    Committed,
}

pub(crate) struct SharedCandidateInputs<'state> {
    pub(super) entries: Mutex<CandidateInputEntries<'state>>,
    pub(super) sharing_enabled: bool,
}

#[derive(Default)]
pub(super) struct CandidateInputEntries<'state> {
    pub(super) entities:
        BTreeMap<(CandidateInputBasis, VersionId, EntityId), Option<VisibleEntityMetadata>>,
    pub(super) relations:
        BTreeMap<(CandidateInputBasis, VersionId, RelationId), Option<VisibleRelationMetadata>>,
    pub(super) entity_aspects: BTreeMap<
        (CandidateInputBasis, VersionId, EntityId),
        Option<&'state AuthoritativeRecordAspectState>,
    >,
    pub(super) relation_aspects: BTreeMap<
        (CandidateInputBasis, VersionId, RelationId),
        Option<&'state AuthoritativeRecordAspectState>,
    >,
    pub(super) adjacency:
        BTreeMap<(CandidateInputBasis, VersionId, EntityId, bool), std::sync::Arc<[RelationId]>>,
    pub(super) adjacency_counts: BTreeMap<(CandidateInputBasis, VersionId, EntityId, bool), usize>,
    pub(super) entity_reads: usize,
    pub(super) relation_reads: usize,
    pub(super) entity_aspect_reads: usize,
    pub(super) relation_aspect_reads: usize,
    pub(super) adjacency_gathers: usize,
    pub(super) adjacency_count_reads: usize,
    pub(super) adjacency_relation_ids: usize,
    pub(super) reuse_hits: usize,
}

impl<'state> SharedCandidateInputs<'state> {
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
                .chain(&access.affected_entity_kinds)
                .copied()
                .collect::<BTreeSet<_>>();
            let rule_relations = access
                .read_relation_kinds
                .iter()
                .chain(&access.affected_relation_kinds)
                .copied()
                .collect::<BTreeSet<_>>();
            if rule_entities.iter().any(|kind| entity_kinds.contains(kind))
                || rule_relations
                    .iter()
                    .any(|kind| relation_kinds.contains(kind))
            {
                sharing_enabled = true;
            }
            entity_kinds.extend(rule_entities);
            relation_kinds.extend(rule_relations);
        }
        (rule_count > 0).then(|| Self {
            entries: Mutex::default(),
            sharing_enabled,
        })
    }

    pub(crate) fn counters(&self) -> (usize, usize, usize, usize, usize, usize) {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (
            entries.entity_reads,
            entries.relation_reads,
            entries.adjacency_gathers,
            entries.adjacency_count_reads,
            entries.adjacency_relation_ids,
            entries.reuse_hits,
        )
    }

    pub(crate) fn aspect_counters(&self) -> (usize, usize) {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (entries.entity_aspect_reads, entries.relation_aspect_reads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::{KindId, PartitionId};

    #[test]
    fn absence_and_foreign_or_changed_basis_do_not_alias() {
        let inputs = SharedCandidateInputs {
            entries: Mutex::default(),
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
        assert_eq!(inputs.counters().0, 3);

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
        assert_eq!(inputs.aspect_counters().0, 3);
    }
}
