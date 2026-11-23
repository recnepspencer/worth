//! A Source's and its Parent's footprint by kind, aspect and field, and the
//! scope entities a probed scope's Source can never reach.
use super::{ManagedIndexParentReadFootprint, ManagedIndexSourceProgram};
use crate::authority::mutation::{
    AdjacencyDelta, AdjacencyDeltaKind, CanonicalAspectDeltaEvidence as Evidence,
    CanonicalRecordAspectDelta, MutationEffect,
};
use crate::derived_relation::DerivedRelationNode as Node;
use crate::graph_member::NativeMemberDomain;
use crate::identity::data::{EntityId, KindId};
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectKey, FieldKey};

/// The literal relations of an ordered-events membership.
struct Ordered {
    predecessor: KindId,
}

pub(super) struct SourceRelevance<'p> {
    program: &'p ManagedIndexSourceProgram,
    footprint: &'p ManagedIndexParentReadFootprint,
    probed: EntityId,
    /// Path membership depends only on stored relations and member lifecycle.
    structural_paths: bool,
    /// The scope kind is only ever the root anchor of a forward stored chain.
    root_only_scope: bool,
    ordered: Option<Ordered>,
    /// Scope entities this candidate created, other than the probed scope.
    born: BTreeSet<EntityId>,
}

impl<'p> SourceRelevance<'p> {
    pub(super) fn of(
        registry: &RelationalSchemaRegistry,
        program: &'p ManagedIndexSourceProgram,
        probed: EntityId,
        footprint: &'p ManagedIndexParentReadFootprint,
        effect: &MutationEffect,
    ) -> Self {
        let scope = program.scope_kind();
        // An ordered-events membership, compiled or behind one derived
        // relation, resolved by the same reader that binds ordered Sources.
        let ordered = program
            .0
            .ordered
            .as_ref()
            .map(|recipe| recipe.predecessor)
            .or_else(|| {
                let main = program.paths().next()?;
                registry.ordered_events_recipe(main).ok().map(|recipe| recipe.predecessor)
            })
            .map(|predecessor| Ordered { predecessor });
        let structural_paths = program.paths().enumerate().all(|(index, path)| {
            let parts = path.parts();
            matches!(parts.from, NativeMemberDomain::Entity(_))
                && matches!(parts.to, NativeMemberDomain::Entity(_))
                && parts.nodes.iter().all(|node| match node {
                    Node::CarrierLift { .. } => false,
                    Node::Derived(_) => index == 0 && ordered.is_some(),
                    _ => true,
                })
        });
        let root_only_scope = structural_paths
            && scope != program.row_kind()
            && program.paths().all(|path| {
                path.parts().nodes.iter().all(|node| {
                    matches!(node, Node::Stored { to, .. } if *to != scope)
                        || matches!(node, Node::Compose { .. })
                })
            })
            && program
                .projection_routes()
                .iter()
                .all(|route| route.endpoint_kind() != scope);
        let born = effect
            .publication
            .canonical_deltas
            .iter()
            .filter_map(|delta| match delta.target {
                RecordRef::Entity(id)
                    if delta.kind_id == scope
                        && id != probed
                        && delta.structural_change == Change::Created =>
                {
                    Some(id)
                }
                _ => None,
            })
            .collect();
        Self {
            program,
            footprint,
            probed,
            structural_paths,
            root_only_scope,
            ordered: ordered.filter(|_| structural_paths),
            born,
        }
    }

    pub(super) fn adjacency(&self, delta: &AdjacencyDelta) -> bool {
        if self.footprint.relations.contains(&delta.kind_id) {
            return true;
        }
        if let Some(ordered) = &self.ordered
            && delta.kind_id == ordered.predecessor
        {
            // Only a scope born here may gain a predecessor: no older chain,
            // the probed one included, can reach a scope created after it.
            return match delta.kind {
                AdjacencyDeltaKind::Created { source, .. } => !self.born.contains(&source),
                AdjacencyDeltaKind::Deleted { .. } => true,
            };
        }
        self.physical(delta.kind_id)
    }

    pub(super) fn delta(&self, delta: &CanonicalRecordAspectDelta) -> bool {
        let kind = delta.kind_id;
        match delta.target {
            RecordRef::Relation(_) => {
                if self.footprint.relations.contains(&kind) {
                    return true;
                }
                match &self.ordered {
                    // Its edge was judged with its endpoints as an adjacency.
                    Some(ordered) if kind == ordered.predecessor => {
                        !matches!(delta.structural_change, Change::Created | Change::Updated)
                    }
                    _ => self.physical(kind),
                }
            }
            RecordRef::Entity(id) => {
                if delta.structural_change == Change::Updated && self.parent_field_changed(delta) {
                    return true;
                }
                if self.other_root(id, kind) {
                    return false;
                }
                if delta.structural_change != Change::Updated
                    && self.footprint.entities.contains(&kind)
                {
                    return true;
                }
                if !self.entity(kind) {
                    return false;
                }
                match delta.structural_change {
                    Change::Updated if self.structural_paths => {
                        delta.contains_opaque_aspect
                            || delta
                                .changed_aspects
                                .iter()
                                .any(|aspect| self.projects(kind, aspect))
                    }
                    _ => true,
                }
            }
        }
    }

    /// A changed record Native described with no aspect delta.
    pub(super) fn undescribed(&self, record: &RecordRef, kind: Option<KindId>) -> bool {
        match (record, kind) {
            (_, None) => true,
            (RecordRef::Entity(id), Some(kind)) => {
                if self.footprint.fields.iter().any(|(field, _, _)| *field == kind) {
                    return true;
                }
                !self.other_root(*id, kind)
                    && (self.entity(kind) || self.footprint.entities.contains(&kind))
            }
            (RecordRef::Relation(_), Some(kind)) => {
                self.physical(kind) || self.footprint.relations.contains(&kind)
            }
        }
    }

    /// A scope entity whose own Source is the only one it can reach.
    fn other_root(&self, entity: EntityId, kind: KindId) -> bool {
        kind == self.program.scope_kind()
            && entity != self.probed
            && (self.root_only_scope || (self.ordered.is_some() && self.born.contains(&entity)))
    }

    fn physical(&self, kind: KindId) -> bool {
        self.program
            .physical_sources()
            .any(|&(relation, _, _)| relation == kind)
    }

    fn entity(&self, kind: KindId) -> bool {
        [self.program.scope_kind(), self.program.row_kind()]
            .into_iter()
            .chain(self.program.physical_sources().flat_map(|&(_, from, to)| [from, to]))
            .any(|candidate| candidate == kind)
    }

    fn projects(&self, kind: KindId, aspect: &AspectKey) -> bool {
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

    /// Whether an update wrote one of the Parent's fields, field by field
    /// where Native kept the old and new values, otherwise by aspect.
    fn parent_field_changed(&self, delta: &CanonicalRecordAspectDelta) -> bool {
        self.footprint
            .fields
            .iter()
            .filter(|(kind, _, _)| *kind == delta.kind_id)
            .any(|(_, aspect, field)| field_changed(delta, aspect, field))
    }
}

fn field_changed(delta: &CanonicalRecordAspectDelta, aspect: &AspectKey, field: &FieldKey) -> bool {
    if delta.contains_opaque_aspect {
        return true;
    }
    let Some(binding) = delta
        .evaluated_bindings
        .iter()
        .find(|binding| binding.aspect_key == *aspect)
    else {
        return delta.changed_aspects.contains(aspect);
    };
    if !binding.changed {
        return false;
    }
    if let Some(changes) = &binding.field_revision_changes {
        return changes.iter().any(|(changed, _)| changed == field);
    }
    match &binding.evidence {
        Evidence::StructAspectValueTransition {
            old_present,
            new_present,
            old_value,
            new_value,
            ..
        } => {
            old_present != new_present
                || old_value.as_ref().and_then(|value| value.get(field))
                    != new_value.as_ref().and_then(|value| value.get(field))
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests;
