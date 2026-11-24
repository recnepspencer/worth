pub(crate) mod applied_candidate_qualification;
pub(crate) mod definition;
pub(crate) mod execution_denial;
pub(crate) mod publication_posture;
pub use definition::{
    DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexId, DerivedIndexKind,
};
pub use execution_denial::{DerivedIndexExecutionDenial, DerivedIndexExecutionDenialKind};
pub use publication_posture::{DerivedIndexApplicability, DerivedIndexPublicationStatus};
mod basis_checkpoint;
pub(crate) use basis_checkpoint::IndexBasisCheckpoint;
mod join_leg_state;
pub(crate) use join_leg_state::{ColdJoinLeg, JoinLegState, JoinLegsState};
mod relation_join_index;
pub use relation_join_index::RelationJoinIndex;
mod installed_routing;
pub(crate) use installed_routing::InstalledIndexRouting;
mod registration_denial;
pub use registration_denial::DerivedIndexRegistrationDenial;
mod entity_field_page;
pub use entity_field_page::{
    BoundedEntityFieldBoundary, BoundedEntityFieldPage, BoundedEntityFieldPageStop,
};
mod related_entity_prefix;
pub use related_entity_prefix::{
    BoundedRelatedEntityPrefixBoundary, BoundedRelatedEntityPrefixDenial,
    BoundedRelatedEntityPrefixLookupRequest, BoundedRelatedEntityPrefixPage,
};
mod selected_read_work;
pub use selected_read_work::SelectedIndexReadWork;
mod allocation_accounting;
mod discard;
pub use discard::{
    DerivedIndexDiscardDenial, DerivedIndexDiscardOutcome, DerivedIndexDiscardRequest,
};
mod entry_map;
pub use entry_map::{DerivedIndexEntryMap, DerivedIndexRows};
pub(crate) mod maintenance;
pub use maintenance::{
    DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind, DerivedIndexMaintenanceWork,
    SelectedIndexGenerationAdmissionStop,
};
mod bounded_entity_field_lookup;
mod bounded_related_entity_ordered_lookup;
mod bounded_relation_join_lookup;
mod generation_residency;
mod generation_selection_counters;
pub use generation_residency::DerivedIndexGenerationResidency;
mod related_entity_ordering;
mod relation_join;

pub use generation_selection_counters::DerivedIndexSelectionCounters;

pub use bounded_entity_field_lookup::{
    BoundedEntityFieldLookupAdmissionStop, BoundedEntityFieldLookupDenial,
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode, MAX_BOUNDED_INDEX_CANDIDATES,
};
pub use bounded_related_entity_ordered_lookup::{
    BoundedRelatedEntityOrderedLookupDenial, BoundedRelatedEntityOrderedLookupDenialKind,
    BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupRequest,
    MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH,
};
pub(crate) use bounded_relation_join_lookup::BoundedRelationJoinLookupWork;
pub use bounded_relation_join_lookup::{
    BoundedRelationJoinLookupDenial, BoundedRelationJoinLookupDenialKind,
    BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupRequest,
};
pub use related_entity_ordering::{
    RelatedEntityEndpoint, RelatedEntityOrderingBoundary, RelatedEntityOrderingDirection,
    RelatedEntityOrderingEntry, RelatedEntityOrderingField, RelatedEntityOrderingValue,
};
pub use relation_join::{
    RelationJoinDefinition, RelationJoinEntry, RelationJoinKey, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};

use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::{BranchId, CommitId};
use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedIndexEntries {
    EntityField(DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, EntityId>),
    RelationField(DerivedIndexEntryMap<AuthoritativeFieldComparisonKey, RelationId>),
    RelatedEntityOrdering(DerivedIndexEntryMap<EntityId, RelatedEntityOrderingEntry>),
    RelationJoin(RelationJoinIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexGeneration {
    pub generation_id: DerivedIndexGenerationId,
    pub index_id: DerivedIndexId,
    pub source_commit_id: CommitId,
    pub source_branch_id: BranchId,
    pub applicability: DerivedIndexApplicability,
    pub status: DerivedIndexPublicationStatus,
    pub entries: DerivedIndexEntries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DerivedIndexArtifacts {
    generations: Vec<DerivedIndexGeneration>,
}

impl DerivedIndexArtifacts {
    pub fn new(generations: Vec<DerivedIndexGeneration>) -> Self {
        let mut artifacts = Self::default();
        artifacts.extend_canonical(&generations);
        artifacts
    }

    pub(crate) fn into_generations(self) -> Vec<DerivedIndexGeneration> {
        self.generations
    }

    pub fn is_empty(&self) -> bool {
        self.generations.is_empty()
    }

    pub fn generations(&self) -> &[DerivedIndexGeneration] {
        &self.generations
    }

    pub fn generation_ids(&self) -> Vec<u64> {
        self.generations
            .iter()
            .map(|generation| generation.generation_id.0)
            .collect()
    }

    pub fn extend_canonical(&mut self, generations: &[DerivedIndexGeneration]) {
        for generation in generations {
            if let Some(existing) = self
                .generations
                .iter_mut()
                .find(|candidate| candidate.generation_id == generation.generation_id)
            {
                *existing = generation.clone();
            } else {
                self.generations.push(generation.clone());
            }
        }
        self.generations
            .sort_by_key(|generation| (generation.index_id.0, generation.generation_id.0));
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.recursive_owned_allocation_capacity_bytes()
    }

    #[cfg(test)]
    pub(crate) fn generations_mut_for_test(&mut self) -> &mut Vec<DerivedIndexGeneration> {
        &mut self.generations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexBuildRequest {
    pub source_commit_id: CommitId,
    pub branch_id: BranchId,
    pub index_ids: Vec<DerivedIndexId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexBuildOutcome {
    pub source_commit_id: CommitId,
    pub generations: Vec<DerivedIndexGeneration>,
    pub failed_indexes: Vec<DerivedIndexId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis_denial: Option<crate::branch::basis_denial::RelationalBranchBasisDenial>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_denial: Option<DerivedIndexExecutionDenial>,
}

mod maintenance_outcome;
pub use maintenance_outcome::DerivedIndexMaintenanceOutcome;
