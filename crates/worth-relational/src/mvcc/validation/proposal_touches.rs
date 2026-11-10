//! Semantic mutation touches projected from one owner-validated plan.
use super::ValidatedRelationalProposal;
use crate::identity::data::{EntityId, KindId, RelationId};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{AspectContract, AspectFieldLocator, FieldKey};
#[path = "proposal_touches/aspect_patch.rs"]
mod aspect_patch;
#[path = "proposal_touches/authored_restoration.rs"]
mod authored_restoration;
#[path = "proposal_touches/collection.rs"]
mod collection;
#[path = "proposal_touches/intent_projection.rs"]
mod intent_projection;
#[path = "proposal_touches/owned_graph.rs"]
mod owned_graph;
#[path = "proposal_touches/preparation.rs"]
mod preparation;
#[path = "proposal_touches/work_observation.rs"]
mod work_observation;

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
    /// A supplied field on a newly created relation; never an update grant.
    InitializeRelationField {
        kind: KindId,
        locator: AspectFieldLocator,
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
            | Self::InitializeRelationField { kind, .. }
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
            .expect(/* Proof: Struct-field groups are constructed only with Some(field); presence-only groups take the separate presence effect path. */ "presence-only groups are unrepresentable, never struct field effects")
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
    AuthoredRestoration(crate::publication::data::PublicationError),
    MissingEntityKind(EntityId),
    MissingRelationKind(RelationId),
    Execution(crate::transactions::data::CommitExecutionDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
}
impl ValidatedRelationalProposal {
    /// Application-authored touches after checking the sealed Native restoration inverse.
    /// Complete mutation inspection remains available through `mutation_touches`.
    pub fn application_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        preparation::project(self, execution, true)
    }

    /// Actor-only authoring in an admitted coupled group. The complete proposal
    /// remains available through `mutation_touches` for Native inspection.
    pub fn authored_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        let commands = match self
            .required_output_publication
            .as_ref()
            .map(|p| &p.publication)
        {
            Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => {
                Some(group.authored_commands.merged_intents.as_slice())
            }
            _ => None,
        };
        preparation::project_commands(self, execution, true, commands)
    }

    /// One concrete producer's touches use its own applied command contribution,
    /// under the final proposal's single invariant judgment and request budgets.
    /// The participant is the part whose producing meaning equals `meaning`
    /// exactly, implementation and epoch included; only that part is projected.
    /// O(parts) meaning comparisons, then O(that part's intents).
    pub fn coupled_producer_mutation_touches<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
    ) -> Option<Result<ValidatedMutationTouches<'a>, ValidatedMutationTouchProjectionError>> {
        let part = self
            .required_output_publication
            .iter()
            .filter_map(|p| match &p.publication {
                crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => Some(group),
                _ => None,
            })
            .flat_map(|group| group.parts.iter())
            .rfind(|part| part.publication.producing_meaning() == Some(meaning))?;
        Some(preparation::project_commands(
            self,
            execution,
            false,
            Some(&part.commands.merged_intents),
        ))
    }

    /// O(reached intents + prior/supplied fields), plus canonical sorting.
    /// Native retained before/after state supplies presence; no history is read.
    pub fn mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        preparation::project(self, execution, false)
    }
}
#[cfg(test)]
#[path = "proposal_touches/tests.rs"]
mod tests;
