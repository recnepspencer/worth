pub(crate) mod completion;
pub(crate) mod source;
use source::NativeMaterializationSource;

use crate::history::data::CommitId;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationalMaterializationRecord {
    Entity {
        entity_id: EntityId,
        kind_id: KindId,
    },
    Relation {
        relation_id: RelationId,
        kind_id: KindId,
        source: EntityId,
        target: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalEntityMaterialization {
    pub entity_id: EntityId,
    pub kind_id: KindId,
    pub fields: AspectFieldPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalRelationMaterialization {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub source: EntityId,
    pub target: EntityId,
    pub fields: AspectFieldPatch,
}

pub struct SuspendRelationalMaterialization;
impl worth_proof::ActionMarker for SuspendRelationalMaterialization {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelationalMaterializationCustodyData {
    pub(crate) branch_identity: crate::branch::identity::RelationalBranchIdentity,
    pub(crate) source_commit: CommitId,
    pub(crate) suspension_commit: CommitId,
    pub(crate) records: std::sync::Arc<Vec<RelationalMaterializationRecord>>,
    pub(crate) source: std::sync::Arc<NativeMaterializationSource>,
    pub(crate) producer_publications: Vec<CommitId>,
    pub(crate) producer_records:
        Vec<crate::history::data::CanonicalRequiredOutputMaterializationRecord>,
}

#[derive(Debug)]
#[must_use = "materialization custody must be retained until reconstruction completes"]
pub struct RelationalMaterializationCustody {
    performed: worth_proof::Performed<
        SuspendRelationalMaterialization,
        super::authority::RelationalBranchMutationAuthorityMarker,
        RelationalMaterializationCustodyData,
    >,
    _producer_memory: worth_execution::ExecutionMemoryReservation,
}

impl RelationalMaterializationCustody {
    pub(crate) fn record(
        data: RelationalMaterializationCustodyData,
        producer_memory: worth_execution::ExecutionMemoryReservation,
    ) -> Self {
        Self {
            _producer_memory: producer_memory,
            performed: worth_proof::Performed::record(
                &crate::branch::authority::issue_relational_branch_mutation_authority(),
                data,
            ),
        }
    }

    pub fn source_commit(&self) -> CommitId {
        self.performed.outcome().source_commit
    }

    pub fn suspension_commit(&self) -> CommitId {
        self.performed.outcome().suspension_commit
    }

    pub fn records(&self) -> &[RelationalMaterializationRecord] {
        &self.performed.outcome().records
    }

    pub(crate) fn data(&self) -> &RelationalMaterializationCustodyData {
        self.performed.outcome()
    }
}
