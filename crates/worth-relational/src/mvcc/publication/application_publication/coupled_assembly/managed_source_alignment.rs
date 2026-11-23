//! Whether the live assembly left one installed managed Source, and the
//! Parent reads beside it, unchanged at a probed scope.
use super::*;
use crate::identity::data::KindId;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::RecordRef;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectKey, FieldKey};
mod relevance;
use relevance::SourceRelevance;

/// What a managed Parent reads beside its Source program: its projected,
/// filtered and ordered fields, its nested result relations and their
/// endpoint kinds, and the scope-key field that resolves the probed scope.
/// A field write is relevant on any entity of its kind, so renaming another
/// scope to the probed key is never hidden.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManagedIndexParentReadFootprint {
    pub fields: Vec<(KindId, AspectKey, FieldKey)>,
    pub relations: Vec<KindId>,
    pub entities: Vec<KindId>,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Whether no record this live assembly changed, through the retained
    /// prefix `pin` reads, can reach `program`'s Source at the probed `scope`
    /// or a read of its Parent's `footprint`.
    ///
    /// A relation change is relevant when its kind is a physical Source or a
    /// Parent relation. An entity of a Source or Parent kind is relevant when
    /// created or removed, and when updated in a projected aspect or a Parent
    /// field. A scope entity other than the probed one may be created,
    /// removed or updated when the scope kind is only the Source's root; under
    /// an ordered-events Source a newly created scope may also be born with a
    /// predecessor edge, which only its own chain can follow. Anything Native
    /// cannot classify is relevant, so `true` is never manufactured.
    /// Cost: O(changed records + deltas + program and footprint size).
    pub fn managed_source_unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        scope: crate::identity::data::EntityId,
        footprint: &ManagedIndexParentReadFootprint,
    ) -> Result<bool, Denial> {
        let current = self.read.borrow();
        let Some(current) = current.as_ref() else {
            return Err(Denial::ForeignSource);
        };
        if !current
            .candidate_identity()
            .includes_assembly_prefix(pin.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        let effect = &self.mutation.outcome.effect;
        let state = &self.mutation.state;
        let registry = self.prepared.schema_authority.registry();
        let relevance = SourceRelevance::of(registry, program, scope, footprint, effect);
        let (unchanged, _) = execution
            .run_native_with_checkpoint(|context| {
                for delta in &effect.adjacency.deltas {
                    checkpoint(context, 1)?;
                    if relevance.adjacency(delta) {
                        return Ok(false);
                    }
                }
                let mut described = BTreeSet::new();
                for delta in &effect.publication.canonical_deltas {
                    checkpoint(context, 1)?;
                    if relevance.delta(delta) {
                        return Ok(false);
                    }
                    described.insert(&delta.target);
                }
                for record in &effect.publication.changed_records {
                    checkpoint(context, 1)?;
                    if described.contains(record) {
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
                crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
            })?;
        unchanged.map_err(|cause: crate::execution::RelationalExecutionDenialCause| {
            Denial::from(crate::derived_relation::DerivedRelationReadDenial::Request(cause))
        })
    }
}
