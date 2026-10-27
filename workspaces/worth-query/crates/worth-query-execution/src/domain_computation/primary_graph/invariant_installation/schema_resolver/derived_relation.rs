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
        if layout.from.borrowed() != relation.from_member()
            || layout.to.borrowed() != relation.to_member()
        {
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
        for (carrier, _) in layout.carriers.iter() {
            if !covered(Target::Entity(carrier.to_string())) {
                return None;
            }
        }
        Some(Binding {
            binding_identity: self.binding_identity.clone(),
            definition: layout.handle.clone(),
            from: self.layout.member_binding(layout.from.borrowed())?,
            to: self.layout.member_binding(layout.to.borrowed())?,
            orbit_definition: layout.orbit_definition,
            _marker: PhantomData,
        })
    }

    /// Resolve a case only with real domain, carrier, read and applicability closure.
    pub fn typed_projected_case<Domain, Carrier, Case>(
        &self,
        domain: worth_query_declaration::facade::application_schema::ApplicationProjectedMemberDomainRef<Schema, Domain, Carrier>,
        case: worth_query_declaration::facade::application_schema::ApplicationProjectedMemberCaseRef<Schema, Domain, Case>,
    ) -> Option<crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantProjectedCaseBinding<Schema, Domain, Carrier, Case>>{
        if case.domain() != domain.name() {
            return None;
        }
        let layout = self.layout.projected_domain(domain.name())?;
        let target = Target::Entity(domain.carrier().to_owned());
        if layout.carrier.as_ref() != domain.carrier()
            || self.layout.entity_kind(domain.carrier()) != Some(layout.handle.parts().carrier)
            || !self.descriptor.read_closure().contains(&target)
            || !self.descriptor.applicability().contains(&target)
        {
            return None;
        }
        Some(crate::domain_computation::primary_graph::application_invariant::WorthQueryApplicationInvariantProjectedCaseBinding {
            binding_identity: self.binding_identity.clone(),
            case: layout.handle.case(case.name())?,
            _marker: PhantomData,
        })
    }
}
