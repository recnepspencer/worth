//! Typed endpoint pairs consume only this exact view's frozen Native source.
use super::{
    WorthQueryApplicationInvariantEntity as Entity, WorthQueryApplicationInvariantReadView as View,
};
#[cfg(test)]
mod native_audience;
use crate::domain_computation::primary_graph::application_invariant::{
    WorthQueryApplicationInvariantDerivedRelationBinding as Binding,
    WorthQueryInvariantAccessDenial as Denial, WorthQueryInvariantAccessDenialKind as Kind,
};
use std::marker::PhantomData;
use worth_relational::facade::runtime::DerivedRelationReadDenial as NativeDenial;

/// A derived endpoint pair has no stored RelationId and grants no mutation authority.
pub struct WorthQueryApplicationInvariantDerivedRelation<Schema, Relation, From, To> {
    from: Entity<Schema, From>,
    to: Entity<Schema, To>,
    _relation: PhantomData<fn() -> Relation>,
}
impl<Schema, Relation, From, To>
    WorthQueryApplicationInvariantDerivedRelation<Schema, Relation, From, To>
{
    pub fn from(&self) -> &Entity<Schema, From> {
        &self.from
    }
    pub fn to(&self) -> &Entity<Schema, To> {
        &self.to
    }
}

impl<Schema> View<'_, Schema> {
    /// Read canonical endpoint pairs derived from the genuine proposed or
    /// committed view. Evaluation refuses roots not prepared in that posture.
    pub fn derived_relations_from<Relation, From, To>(
        &self,
        binding: &Binding<Schema, Relation, From, To>,
        root: &Entity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantDerivedRelation<Schema, Relation, From, To>>,
        Denial,
    > {
        self.check_binding(&binding.binding_identity)?;
        self.check_entity_affinity(root, binding.definition.parts().from)?;
        let key = self
            .relations
            .derived_read_identity(&binding.definition, root.entity_id)
            .map_err(native_denial)?;
        let (prepared, acquired) =
            self.admission
                .derived(key, &self.relations, &binding.definition, root.entity_id)?;
        let native = self
            .relations
            .read_prepared_derived_relations(&prepared)
            .map_err(native_denial)?;
        if acquired {
            self.reads.derived(native.reads());
        }
        for (entity, _) in native.entity_dependencies() {
            self.admission
                .entity(*entity, self.relations.read_budget())?;
        }
        for record in native.physical_relations() {
            self.admission
                .relation(record.relation_id, self.relations.read_budget())?;
        }
        self.relations
            .read_budget()
            .claim_retained(
                (native.endpoints().len() as u64).saturating_mul(size_of::<
                    WorthQueryApplicationInvariantDerivedRelation<Schema, Relation, From, To>,
                >() as u64),
            )
            .map_err(|cause| Denial::from_native_read(cause, "derived endpoint result"))?;
        let mut result = Vec::with_capacity(native.endpoints().len());
        for endpoint in native.endpoints() {
            self.relations
                .read_budget()
                .checkpoint(1)
                .map_err(|cause| Denial::from_native_read(cause, "derived endpoint result"))?;
            result.push(WorthQueryApplicationInvariantDerivedRelation {
                from: root.clone(),
                to: self.entity(*endpoint, binding.definition.parts().to),
                _relation: PhantomData,
            });
        }
        self.relations
            .read_budget()
            .check_live()
            .map_err(|cause| Denial::from_native_read(cause, "derived endpoint result"))?;
        Ok(result)
    }
}

pub(in crate::domain_computation::primary_graph::application_invariant) fn native_denial(
    cause: NativeDenial,
) -> Denial {
    use worth_relational::facade::schema::DerivedRelationDenial as Definition;
    match cause {
        NativeDenial::Structural(cause) => Denial::from_native_read(cause, "derived relation read"),
        NativeDenial::Request(cause) => {
            Denial::new(Kind::Execution(cause), "derived relation read")
        }
        NativeDenial::Definition(cause) => {
            let kind = match &cause {
                Definition::ForeignObservation => Kind::ForeignView,
                Definition::ForeignDefinition => Kind::ForeignBinding,
                Definition::OutsideDeclaredAccess => Kind::OutsideDeclaredAccess,
                Definition::RecordUnavailable => Kind::EntityUnavailable,
                Definition::DiscontinuousEndpoints => Kind::WrongEntityKind,
                Definition::WorkBudgetExceeded => Kind::WorkBudgetExceeded,
                Definition::InvalidIdentity
                | Definition::DuplicateDefinition(_)
                | Definition::MissingDefinition(_)
                | Definition::CyclicDefinitions
                | Definition::InvalidExpression
                | Definition::InvalidPhysicalSource(_) => Kind::RelationNotInstalled,
            };
            Denial::new(kind, format!("derived relation read: {cause:?}"))
        }
    }
}

#[cfg(test)]
mod native_fixture;
#[cfg(test)]
mod native_snapshot;
