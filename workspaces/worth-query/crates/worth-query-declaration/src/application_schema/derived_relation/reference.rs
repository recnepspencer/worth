//! Typed names identify declared meaning, not physical relation records.
use crate::application_schema::{
    ApplicationGraphMemberSelector as Member, ApplicationProjectedMemberDomainRef,
};
use std::marker::PhantomData;

/// A named derived relation with exact typed endpoint kinds.
pub struct ApplicationDerivedRelationRef<Schema, Relation, From, To> {
    name: &'static str,
    from: Member<'static>,
    to: Member<'static>,
    marker: PhantomData<fn() -> (Schema, Relation, From, To)>,
}

impl<S, R, F, T> Copy for ApplicationDerivedRelationRef<S, R, F, T> {}
impl<S, R, F, T> Clone for ApplicationDerivedRelationRef<S, R, F, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, R, F, T> ApplicationDerivedRelationRef<S, R, F, T> {
    /// Records declared names; completed-schema validation checks their closure.
    pub const fn from_schema_identifiers(
        name: &'static str,
        from: &'static str,
        to: &'static str,
    ) -> Self {
        Self {
            name,
            from: Member::Entity(from),
            to: Member::Entity(to),
            marker: PhantomData,
        }
    }

    /// The declared derived relation name.
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The declared source entity kind.
    pub const fn from(self) -> &'static str {
        self.from.name()
    }

    /// The declared target entity kind.
    pub const fn to(self) -> &'static str {
        self.to.name()
    }

    /// Exact endpoint namespaces; names alone do not erase projected meaning.
    pub const fn from_member(self) -> Member<'static> {
        self.from
    }
    pub const fn to_member(self) -> Member<'static> {
        self.to
    }

    /// Records explicit endpoint namespaces for fresh completed-schema validation.
    pub const fn from_member_identifiers(
        name: &'static str,
        from: Member<'static>,
        to: Member<'static>,
    ) -> Self {
        Self {
            name,
            from,
            to,
            marker: PhantomData,
        }
    }
}

impl<S, R, D> ApplicationDerivedRelationRef<S, R, D, D> {
    /// Names a homogeneous relation over an actual finite declared domain.
    pub const fn projected<C>(
        name: &'static str,
        domain: ApplicationProjectedMemberDomainRef<S, D, C>,
    ) -> Self {
        Self::from_member_identifiers(
            name,
            Member::Projected(domain.name()),
            Member::Projected(domain.name()),
        )
    }
}
