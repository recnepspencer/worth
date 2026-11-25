//! Judge a selected Source against the exact native effects and the applied image.
use super::super::parent_read_footprint::ManagedIndexParentReadFootprint;
use super::reach::Reach;
use super::relevance::SourceRelevance;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::transactions::data::primitives::RecordRef;
use contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use contracts::validation::candidate_source::program::ManagedIndexSourceProgram;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use std::cell::RefCell;

type Read = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// Live inputs to both managed Source stability judgments.
pub struct Assessment<'a> {
    pub current: &'a RefCell<Option<Read>>,
    pub registry: &'a RelationalSchemaRegistry,
    pub state: &'a WorkingState,
    pub effect: &'a MutationEffect,
}

impl Assessment<'_> {
    pub fn unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &Read,
        scope: EntityId,
        footprint: &ManagedIndexParentReadFootprint,
    ) -> Result<bool, Denial> {
        self.unchanged(execution, program, pin, scope, footprint, None, None)
    }

    /// Use the kind-level proof first. A relevant member is narrowed against
    /// the membership read in its exact preceding image. Requests may stop
    /// the judgment, while an unreadable membership leaves the kind-level answer.
    pub fn unchanged_by(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &Read,
        earlier: &Read,
        scope: EntityId,
        footprint: &ManagedIndexParentReadFootprint,
        records: &[RecordRef],
        prior_reach: impl FnOnce() -> Result<Reach, Denial>,
    ) -> Result<bool, Denial> {
        if self.unchanged(
            execution,
            program,
            pin,
            scope,
            footprint,
            Some(records),
            None,
        )? {
            return Ok(true);
        }
        if !pin
            .candidate_identity()
            .includes_assembly_prefix(earlier.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        match prior_reach() {
            Ok(reach) => self.unchanged(
                execution,
                program,
                pin,
                scope,
                footprint,
                Some(records),
                Some(reach),
            ),
            Err(
                cause @ Denial::Source(
                    contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(_),
                ),
            ) => Err(cause),
            Err(_) => Ok(false),
        }
    }

    fn unchanged(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &Read,
        scope: EntityId,
        footprint: &ManagedIndexParentReadFootprint,
        only: Option<&[RecordRef]>,
        reach: Option<Reach>,
    ) -> Result<bool, Denial> {
        let current = self.current.borrow();
        let Some(current) = current.as_ref() else {
            return Err(Denial::ForeignSource);
        };
        if !current
            .candidate_identity()
            .includes_assembly_prefix(pin.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        source_is_unchanged(
            execution,
            program,
            scope,
            footprint,
            self.registry,
            self.state,
            self.effect,
            only,
            reach,
        )
    }
}

fn source_is_unchanged(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    footprint: &ManagedIndexParentReadFootprint,
    registry: &RelationalSchemaRegistry,
    state: &WorkingState,
    effect: &MutationEffect,
    only: Option<&[RecordRef]>,
    reach: Option<Reach>,
) -> Result<bool, Denial> {
    let key_of = |entity: EntityId| scope_key_value(state, footprint.scope_key.as_ref()?, entity);
    let relevance =
        SourceRelevance::of(registry, program, scope, footprint, effect, &key_of).reached_by(reach);
    let (unchanged, _) = execution
        .run_native_with_checkpoint(|context| {
            let only = match only {
                Some(records) => {
                    checkpoint(context, records.len() as u64 + 1)?;
                    Some(records.iter().collect::<BTreeSet<_>>())
                }
                None => None,
            };
            let judged =
                |record: &RecordRef| only.as_ref().is_none_or(|only| only.contains(record));
            for delta in &effect.adjacency.deltas {
                checkpoint(context, 1)?;
                if judged(&RecordRef::Relation(delta.relation_id)) && relevance.adjacency(delta) {
                    return Ok(false);
                }
            }
            let mut described = BTreeSet::new();
            for delta in &effect.publication.canonical_deltas {
                checkpoint(context, 1)?;
                if judged(&delta.target) && relevance.delta(delta) {
                    return Ok(false);
                }
                described.insert(&delta.target);
            }
            for record in &effect.publication.changed_records {
                checkpoint(context, 1)?;
                if described.contains(record) || !judged(record) {
                    continue;
                }
                let kind = match record {
                    RecordRef::Entity(id) => state
                        .get_current_partition(id.partition_id)
                        .and_then(|partition| partition.entity(*id))
                        .and_then(|slot| slot.kind_id()),
                    RecordRef::Relation(id) => state
                        .get_current_partition(id.partition_id)
                        .and_then(|partition| partition.relation(*id))
                        .and_then(|slot| slot.kind_id()),
                };
                if relevance.undescribed(record, kind) {
                    return Ok(false);
                }
            }
            Ok(true)
        })
        .map_err(|cause| {
            contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(
                cause.into(),
            )
        })?;
    unchanged.map_err(
        |cause: contracts::execution::denial_cause::RelationalExecutionDenialCause| {
            Denial::from(
                contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(cause),
            )
        },
    )
}

/// One entity's scope-key value in the candidate's working state, when it is
/// an entity of the key's kind with that field present.
fn scope_key_value(
    state: &WorkingState,
    (kind, aspect, field): &(KindId, AspectKey, FieldKey),
    entity: EntityId,
) -> Option<worth_foundational::facade::AspectValue> {
    use worth_foundational::facade::ContractValidatedAspectValueView as View;
    let partition = state.get_current_partition(entity.partition_id)?;
    let slot = partition.entity(entity)?;
    if slot.kind_id()? != *kind {
        return None;
    }
    match slot
        .extra()
        .authoritative_aspect_state()?
        .get(aspect)?
        .view()
    {
        View::Struct(value) => value.get(field).cloned(),
        View::Scalar(value) => Some(value.clone()),
    }
}
