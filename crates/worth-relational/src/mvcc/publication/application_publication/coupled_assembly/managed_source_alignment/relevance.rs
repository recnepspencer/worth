//! A Source's and its Parent's footprint by kind, aspect and field, and the
//! scope entities a probed scope's Source can never reach.
use super::super::parent_read_footprint::ManagedIndexParentReadFootprint;
use crate::authority::mutation::{
    AdjacencyDelta, AdjacencyDeltaKind, CanonicalAspectDeltaEvidence as Evidence,
    CanonicalRecordAspectDelta, MutationEffect,
};
use crate::derived_relation::DerivedRelationNode as Node;
use crate::graph_member::NativeMemberDomain;
use crate::identity::data::{EntityId, KindId};
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectKey, FieldKey};

/// The literal relations of an ordered-events membership.
#[derive(Clone, Copy)]
struct Ordered {
    predecessor: KindId,
    introduced: KindId,
    retired: KindId,
}

/// Reads one entity's scope-key value from the candidate.
pub(super) type ScopeKeyReader<'p> =
    &'p dyn Fn(EntityId) -> Option<worth_foundational::facade::AspectValue>;

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
    key_of: ScopeKeyReader<'p>,
    probed_key: Option<worth_foundational::facade::AspectValue>,
}

impl<'p> SourceRelevance<'p> {
    pub(super) fn of(
        registry: &RelationalSchemaRegistry,
        program: &'p ManagedIndexSourceProgram,
        probed: EntityId,
        footprint: &'p ManagedIndexParentReadFootprint,
        effect: &MutationEffect,
        key_of: ScopeKeyReader<'p>,
    ) -> Self {
        let scope = program.scope_kind();
        // An ordered-events membership, compiled or behind one derived
        // relation, resolved by the same reader that binds ordered Sources.
        let ordered = program
            .0
            .ordered
            .as_ref()
            .map(|recipe| (recipe.predecessor, recipe.introduced, recipe.retired))
            .or_else(|| {
                let main = program.paths().next()?;
                let recipe = registry.ordered_events_recipe(main).ok()?;
                Some((recipe.predecessor, recipe.introduced, recipe.retired))
            })
            .map(|(predecessor, introduced, retired)| Ordered {
                predecessor,
                introduced,
                retired,
            });
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
        // A born scope's predecessor is followed only by its own chain while
        // the scope is never a row, the order relation is no membership event,
        // and no projection route reads a scope or the order relation.
        let ordered = ordered.filter(|recipe| {
            structural_paths
                && scope != program.row_kind()
                && recipe.predecessor != recipe.introduced
                && recipe.predecessor != recipe.retired
                && program
                    .projection_routes()
                    .iter()
                    .all(|route| route.endpoint_kind() != scope)
                && program.paths().skip(1).all(|route| {
                    route
                        .physical_sources()
                        .iter()
                        .all(|&(relation, _, _)| relation != recipe.predecessor)
                })
        });
        Self {
            program,
            footprint,
            probed,
            structural_paths,
            root_only_scope,
            ordered,
            born,
            key_of,
            probed_key: key_of(probed),
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
                // A Parent-read kind's lifecycle is relevant on any entity.
                if delta.structural_change != Change::Updated
                    && self.footprint.entities.contains(&kind)
                {
                    return true;
                }
                if self.other_root(id, kind) {
                    return self.appears_with_probed_key(delta, id);
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
                if self
                    .footprint
                    .fields
                    .iter()
                    .any(|(field, _, _)| *field == kind)
                {
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

    /// A scope that appears in the candidate answering to the probed scope's
    /// key would make that key name another scope: equal, unreadable or
    /// opaque keys are relevant; a distinct key is not.
    fn appears_with_probed_key(&self, delta: &CanonicalRecordAspectDelta, id: EntityId) -> bool {
        if !matches!(
            delta.structural_change,
            Change::Created | Change::Rematerialized
        ) || self.footprint.scope_key.is_none()
        {
            return false;
        }
        if delta.contains_opaque_aspect {
            return true;
        }
        match (&self.probed_key, (self.key_of)(id)) {
            (Some(probed), Some(sibling)) => *probed == sibling,
            _ => true,
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
            .chain(
                self.program
                    .physical_sources()
                    .flat_map(|&(_, from, to)| [from, to]),
            )
            .any(|candidate| candidate == kind)
    }

    fn projects(&self, kind: KindId, aspect: &AspectKey) -> bool {
        let projected =
            |fields: &[crate::validation::candidate_source::ManagedIndexSourceField]| {
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
