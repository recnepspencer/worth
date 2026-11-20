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
            .state
            .required_output_publication
            .as_ref()
            .map(|p| &p.publication)
        {
            Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => {
                Some(group.authored_commands.plan().merged_intents.as_slice())
            }
            _ => None,
        };
        preparation::project_commands(self, execution, true, commands)
    }

    /// Actor-only authoring of one later authored stage (`stage` 0 is the
    /// first stage after the prefix), from that stage's own applied commands.
    /// Each stage's invariants are judged on its own contribution, as each
    /// producer's are; `None` when the group has no such stage.
    pub fn authored_stage_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        stage: usize,
    ) -> Option<Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError>> {
        let part = match self
            .state
            .required_output_publication
            .as_ref()
            .map(|p| &p.publication)
        {
            Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => {
                group.stages.get(stage)?
            }
            _ => return None,
        };
        Some(preparation::project_commands(
            self,
            execution,
            true,
            Some(&part.commands.plan().merged_intents),
        ))
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
            .state
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
            Some(&part.commands.plan().merged_intents),
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

/// Project semantic touches directly from an applied suffix's normalized intents
/// and current working state, charged to `execution` (PR C append-time touches).
pub(crate) fn project_applied_intents(
    execution: &ExecutionRequest<'_, '_>,
    state: &crate::storage::overlay::WorkingState,
    intents: &[crate::transactions::data::MutationIntent],
) -> Result<Vec<ValidatedMutationTouch>, crate::transactions::data::TransactionCommitError> {
    use crate::execution::request_stage::checkpoint;
    use crate::transactions::data::{EntityMutationIntent, MutationIntent};

    let map_err = |err: ValidatedMutationTouchProjectionError| match err {
        ValidatedMutationTouchProjectionError::Execution(cause) => {
            crate::transactions::data::TransactionCommitError::execution(cause)
        }
        ValidatedMutationTouchProjectionError::Allocation(denial) => {
            crate::transactions::data::TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::ExecutionAllocationDenied { denial },
                ),
            )
        }
        other => crate::mvcc::origin_denial(format!("applied touch projection failed: {other:?}")),
    };
    let mut touches = Vec::new();
    execution
        .run_native_with_checkpoint(|context| {
            let mut charge = |units: u64| {
                checkpoint(context, units).map_err(|stop| collection::execution(stop.into()))
            };
            let mut sink = collection::TouchCollection {
                extent: collection::Extent::default(),
                backing: Some(TouchBacking {
                    touches: Vec::new(),
                    structs: Vec::new(),
                    _memory: None,
                }),
                failure: None,
                checkpoint: &mut charge,
            };
            let mut work = ValidatedMutationTouchProjectionWork::default();
            for intent in intents {
                (sink.checkpoint)(1).map_err(map_err)?;
                match intent {
                    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
                        let kind = intent_projection::entity_kind(state, spec.entity_id, &mut work)
                            .map_err(map_err)?;
                        intent_projection::project_entity_aspect_patch(
                            kind,
                            &spec.aspect_patch,
                            &mut sink,
                        );
                    }
                    _ => intent_projection::project_intent(intent, state, &mut sink, &mut work)
                        .map_err(map_err)?,
                }
                if let Some(failure) = sink.failure.take() {
                    return Err(map_err(failure));
                }
            }
            let backing = sink.backing.expect("owned touch backing");
            touches = backing.touches;
            let comparisons = touches.len().saturating_mul(
                usize::BITS as usize - touches.len().max(1).leading_zeros() as usize,
            );
            checkpoint(context, comparisons as u64 + 1)
                .map_err(crate::mvcc::publication::custody_stop)?;
            touches.sort_unstable();
            touches.dedup();
            Ok::<_, crate::transactions::data::TransactionCommitError>(())
        })
        .map_err(crate::mvcc::publication::custody_stop)?
        .0?;
    Ok(touches)
}
#[cfg(test)]
#[path = "proposal_touches/tests.rs"]
mod tests;
