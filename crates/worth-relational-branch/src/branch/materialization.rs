pub mod source;
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
pub struct RelationalMaterializationCustodyData {
    pub branch_identity: crate::branch::identity::RelationalBranchIdentity,
    pub source_commit: CommitId,
    pub suspension_commit: CommitId,
    pub records: std::sync::Arc<Vec<RelationalMaterializationRecord>>,
    pub source: std::sync::Arc<NativeMaterializationSource>,
    pub producer_publications: Vec<CommitId>,
    pub producer_records: Vec<crate::history::data::CanonicalRequiredOutputMaterializationRecord>,
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
    pub fn source_commit(&self) -> CommitId {
        self.performed.outcome().source_commit
    }

    pub fn suspension_commit(&self) -> CommitId {
        self.performed.outcome().suspension_commit
    }

    pub fn records(&self) -> &[RelationalMaterializationRecord] {
        &self.performed.outcome().records
    }
}

/// Operations used by the Relational owners.
pub trait RelationalMaterializationCustodyOwnerPort: Sized {
    fn record(
        data: RelationalMaterializationCustodyData,
        producer_memory: worth_execution::ExecutionMemoryReservation,
    ) -> Self;
    fn data(&self) -> &RelationalMaterializationCustodyData;
}

impl RelationalMaterializationCustodyOwnerPort for RelationalMaterializationCustody {
    fn record(
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

    fn data(&self) -> &RelationalMaterializationCustodyData {
        self.performed.outcome()
    }
}
