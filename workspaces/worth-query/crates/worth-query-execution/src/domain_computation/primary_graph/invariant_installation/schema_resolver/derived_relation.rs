//! A derived binding admits its entire physical dependency closure.
use crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantDerivedRelationBinding as Binding;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationInvariantScopeTarget as Target,
};

impl<Schema> super::WorthQueryApplicationInvariantSchemaResolver<'_, Schema> {
    /// Resolve a genuine installed recipe only when reads AND applicability
    /// include every physical leg and endpoint kind, including empty results.
    pub fn typed_derived_relation<Relation, From, To>(
        &self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
    ) -> Option<Binding<Schema, Relation, From, To>> {
        let layout = self.layout.derived_relation(relation.name())?;
        if layout.from.as_ref() != relation.from() || layout.to.as_ref() != relation.to() {
            return None;
        }
        let covered = |target: Target| {
            self.descriptor.read_closure().contains(&target)
                && self.descriptor.applicability().contains(&target)
        };
        for (name, _, from, to) in layout.sources.iter() {
            if !covered(Target::Relation(name.to_string()))
                || !covered(Target::Entity(self.layout.entity_name(*from)?.to_owned()))
                || !covered(Target::Entity(self.layout.entity_name(*to)?.to_owned()))
            {
                return None;
            }
        }
        Some(Binding {
            binding_identity: self.binding_identity.clone(),
            definition: layout.handle.clone(),
            _marker: PhantomData,
        })
    }
}
