//! Semantic mutation touches projected from one owner-validated plan.
use super::ValidatedRelationalProposal;
use crate::identity::data::{EntityId, KindId, RelationId};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{AspectContract, AspectFieldLocator, FieldKey};
#[path = "proposal_touches/collection.rs"]
mod collection;
#[path = "proposal_touches/intent_projection.rs"]
mod intent_projection;
#[path = "proposal_touches/preparation.rs"]
mod preparation;
#[path = "proposal_touches/struct_aspect.rs"]
mod struct_aspect;

/// One semantic locus touched by an invariant-validated Relational mutation.
///
/// The two `Unrepresentable` variants preserve owner truth for mutation forms
/// that cannot honestly be admitted as an exact field or graph-edge touch.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidatedMutationTouch {
    CreateEntity {
        kind: KindId,
    },
    DeleteEntity {
        kind: KindId,
    },
    WriteEntityField {
        kind: KindId,
        locator: AspectFieldLocator,
    },
    LinkRelation {
        kind: KindId,
    },
    UnlinkRelation {
        kind: KindId,
    },
    UnrepresentableEntityMutation {
        kind: KindId,
    },
    /// One unchanged record brought back under judgement. It is not a write:
    /// the record's stored state is exactly what it was before the candidate.
    RevalidateEntity {
        kind: KindId,
    },
    /// One unchanged relation brought back under judgment, without an edge write.
    RevalidateRelation {
        kind: KindId,
    },
    UnrepresentableRelationMutation {
        kind: KindId,
    },
}

impl ValidatedMutationTouch {
    pub const fn kind(&self) -> KindId {
        match self {
            Self::CreateEntity { kind }
            | Self::DeleteEntity { kind }
            | Self::WriteEntityField { kind, .. }
            | Self::LinkRelation { kind }
            | Self::UnlinkRelation { kind }
            | Self::UnrepresentableEntityMutation { kind }
            | Self::RevalidateEntity { kind }
            | Self::RevalidateRelation { kind }
            | Self::UnrepresentableRelationMutation { kind } => *kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidatedMutationTouchProjectionWork {
    validated_intents_examined: usize,
    mutation_targets_materialized: usize,
    owner_state_lookups: usize,
}

impl ValidatedMutationTouchProjectionWork {
    pub const fn validated_intents_examined(self) -> usize {
        self.validated_intents_examined
    }

    pub const fn mutation_targets_materialized(self) -> usize {
        self.mutation_targets_materialized
    }

    pub const fn owner_state_lookups(self) -> usize {
        self.owner_state_lookups
    }
}

/// A whole-struct effect on one actual prior or supplied field. Borrowing the
/// candidate's admitted contract preserves its full meaning without copying it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedEntityStructTouch<'proposal> {
    kind: KindId,
    contract: &'proposal AspectContract,
    entity: EntityId,
    field: Option<&'proposal FieldKey>,
    before_present: bool,
    after_present: bool,
}
impl ValidatedEntityStructTouch<'_> {
    pub const fn kind(&self) -> KindId {
        self.kind
    }
    pub const fn contract(&self) -> &AspectContract {
        self.contract
    }
    pub fn field(&self) -> &FieldKey {
        self.field
            .expect("presence-only groups are unrepresentable, never struct field effects")
    }
    pub const fn before_present(&self) -> bool {
        self.before_present
    }
    pub const fn after_present(&self) -> bool {
        self.after_present
    }
}

#[derive(Debug)]
struct TouchBacking<'proposal> {
    touches: Vec<ValidatedMutationTouch>,
    structs: Vec<ValidatedEntityStructTouch<'proposal>>,
    // Last to drop: all returned allocation and nested locator charges survive
    // every shared observer and are released only after their owned payload.
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Clone, Debug)]
pub struct ValidatedMutationTouches<'proposal> {
    backing: Arc<TouchBacking<'proposal>>,
    work: ValidatedMutationTouchProjectionWork,
}
impl ValidatedMutationTouches<'_> {
    pub fn touches(&self) -> &[ValidatedMutationTouch] {
        &self.backing.touches
    }
    pub fn struct_touches(&self) -> &[ValidatedEntityStructTouch<'_>] {
        &self.backing.structs
    }
    pub const fn work(&self) -> ValidatedMutationTouchProjectionWork {
        self.work
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidatedMutationTouchProjectionError {
    MissingEntityKind(EntityId),
    MissingRelationKind(RelationId),
    Execution(crate::transactions::data::CommitExecutionDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
}
impl ValidatedRelationalProposal {
    /// O(reached intents + prior/supplied fields), plus canonical sorting.
    /// Native retained before/after state supplies presence; no history is read.
    pub fn mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        preparation::project(self, execution)
    }
}
#[cfg(test)]
#[path = "proposal_touches/tests.rs"]
mod tests;
