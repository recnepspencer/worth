//! Canonical provenance of a payload selected from an actual retained root.
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchTarget, RelationalBranchVersion,
    CANONICAL_RUNTIME_INSTANCE_ID,
};
use crate::history::data::BranchId;
use crate::identity::data::{EntityId, KindId};
use serde::{Deserialize, Serialize};
use worth_foundational::FoundationalBranchReferenceGeneration;

/// Describes the historical payload of a fresh selection occurrence.
/// This is provenance, never a source lease or permission to select again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRetainedEntitySelection {
    source_branch: BranchId,
    source_generation: FoundationalBranchReferenceGeneration,
    source_truth_version: RelationalBranchVersion,
    source_head: RelationalBranchTarget,
    entity: EntityId,
    kind: KindId,
}

impl CanonicalRetainedEntitySelection {
    pub(crate) fn from_admitted(
        source: &AdmittedRelationalBranchBasis,
        entity: EntityId,
        kind: KindId,
    ) -> Option<Self> {
        let worth_foundational::FoundationalBranchTarget::Basis(target) =
            source.reference().target()
        else {
            return None;
        };
        Some(Self {
            source_branch: source.identity().branch_id().clone(),
            source_generation: source.reference().generation(),
            source_truth_version: source.truth_version(),
            source_head: target.rebind_runtime_instance_id(CANONICAL_RUNTIME_INSTANCE_ID),
            entity,
            kind,
        })
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub const fn source_generation(&self) -> FoundationalBranchReferenceGeneration {
        self.source_generation
    }

    pub const fn source_truth_version(&self) -> RelationalBranchVersion {
        self.source_truth_version
    }

    pub fn source_head(&self) -> &RelationalBranchTarget {
        &self.source_head
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn kind(&self) -> KindId {
        self.kind
    }
}
