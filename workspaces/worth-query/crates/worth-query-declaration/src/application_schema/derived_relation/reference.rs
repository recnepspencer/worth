//! Typed names identify declared meaning, not physical relation records.
use std::marker::PhantomData;

/// A named derived relation with exact typed endpoint kinds.
pub struct ApplicationDerivedRelationRef<Schema, Relation, From, To> {
    name: &'static str,
    from: &'static str,
    to: &'static str,
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
            from,
            to,
            marker: PhantomData,
        }
    }

    /// The declared derived relation name.
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The declared source entity kind.
    pub const fn from(self) -> &'static str {
        self.from
    }

    /// The declared target entity kind.
    pub const fn to(self) -> &'static str {
        self.to
    }
}
