//! Whether the live assembly left one installed managed Source unchanged.
use super::*;
use crate::authority::mutation::CanonicalRecordAspectDelta;
use crate::derived_relation::DerivedRelationNode as Node;
use crate::graph_member::NativeMemberDomain;
use crate::identity::data::KindId;
use crate::publication::patch::data::RecordStructuralChange;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::RecordRef;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Whether no record this live assembly changed, through the retained
    /// prefix `pin` reads, can reach `program`'s Source, for every scope.
    ///
    /// A relation change is relevant when its kind is a physical source of
    /// the program. An entity of the program's scope, row or path-endpoint
    /// kind is relevant when created or removed, and when updated in an aspect
    /// the Source projects (a row field, or a projection route's field). When
    /// a path could depend on member content (a lifted or derived step), or
    /// a changed record has no canonical aspect delta, Native's coarse kind
    /// relevance applies instead. `true` is therefore never manufactured.
    /// Cost: O(changed records + adjacency deltas + program size).
    pub fn managed_source_unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
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
        let relevance = SourceRelevance::of(program);
        let (unchanged, _) = execution
            .run_native_with_checkpoint(|context| {
                for delta in &effect.adjacency.deltas {
                    checkpoint(context, 1)?;
                    if relevance.relation(delta.kind_id) {
                        return Ok(false);
                    }
                }
                for delta in &effect.publication.canonical_deltas {
                    checkpoint(context, 1)?;
                    if relevance.delta(delta) {
                        return Ok(false);
                    }
                }
                for record in &effect.publication.changed_records {
                    checkpoint(context, 1)?;
                    if effect
                        .publication
                        .canonical_deltas
                        .iter()
                        .any(|delta| delta.target == *record)
                    {
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
                    let relevant = match (record, kind) {
                        (_, None) => true,
                        (RecordRef::Entity(_), Some(kind)) => relevance.entity(kind),
                        (RecordRef::Relation(_), Some(kind)) => relevance.relation(kind),
                    };
                    if relevant {
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

/// The program's Source footprint, by kind and projected aspect.
struct SourceRelevance<'p> {
    program: &'p ManagedIndexSourceProgram,
    /// Path membership depends only on stored relations and member lifecycle.
    structural_paths: bool,
}

impl<'p> SourceRelevance<'p> {
    fn of(program: &'p ManagedIndexSourceProgram) -> Self {
        let structural_paths = program.paths().all(|path| {
            let parts = path.parts();
            matches!(parts.from, NativeMemberDomain::Entity(_))
                && matches!(parts.to, NativeMemberDomain::Entity(_))
                && parts.nodes.iter().all(|node| {
                    !matches!(node, Node::Derived(_) | Node::CarrierLift { .. })
                })
        });
        Self {
            program,
            structural_paths,
        }
    }
    fn relation(&self, kind: KindId) -> bool {
        self.program
            .physical_sources()
            .any(|&(relation, _, _)| relation == kind)
    }
    fn entity(&self, kind: KindId) -> bool {
        [self.program.scope_kind(), self.program.row_kind()]
            .into_iter()
            .chain(
                self.program
                    .physical_sources()
                    .flat_map(|&(_, from, to)| [from, to]),
            )
            .any(|candidate| candidate == kind)
    }
    fn projects(&self, kind: KindId, aspect: &worth_foundational::facade::AspectKey) -> bool {
        let projected = |fields: &[crate::validation::candidate_source::ManagedIndexSourceField]| {
            fields
                .iter()
                .any(|field| field.locator.aspect().aspect_key() == aspect)
        };
        (kind == self.program.row_kind() && projected(self.program.fields()))
            || self
                .program
                .projection_routes()
                .iter()
                .any(|route| route.endpoint_kind() == kind && projected(route.fields()))
    }
    fn delta(&self, delta: &CanonicalRecordAspectDelta) -> bool {
        match delta.target {
            RecordRef::Relation(_) => self.relation(delta.kind_id),
            RecordRef::Entity(_) if !self.entity(delta.kind_id) => false,
            RecordRef::Entity(_) => match delta.structural_change {
                RecordStructuralChange::Updated if self.structural_paths => {
                    delta.contains_opaque_aspect
                        || delta
                            .changed_aspects
                            .iter()
                            .any(|aspect| self.projects(delta.kind_id, aspect))
                }
                _ => true,
            },
        }
    }
}
