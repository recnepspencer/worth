use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe,
    ApplicationFieldMarkerIdentity, ApplicationFieldUnit, ApplicationRelationRef,
    NoApplicationUnit, ReadOnly,
};

/// A field on one actual declared relation, including its typed endpoint kinds.
pub struct ApplicationRelationFieldRef<
    Schema,
    Relation,
    From,
    To,
    Aspect,
    Field,
    Write = ReadOnly,
    Unit = NoApplicationUnit,
> {
    relation: ApplicationRelationRef<Schema, Relation, From, To>,
    _field: PhantomData<fn() -> (Aspect, Field, Write, Unit)>,
}

impl<Schema, Relation, From, To, Aspect, Field, Write, Unit> Copy
    for ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>
{
}

impl<Schema, Relation, From, To, Aspect, Field, Write, Unit> Clone
    for ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Relation, From, To, Aspect, Field, Write, Unit>
    ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>
where
    Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
    Field: ApplicationFieldMarkerIdentity<Schema, Relation, Aspect>,
    Unit: ApplicationFieldUnit,
{
    pub const fn from_relation(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self {
            relation,
            _field: PhantomData,
        }
    }

    pub const fn relation(&self) -> ApplicationRelationRef<Schema, Relation, From, To> {
        self.relation
    }
    pub const fn aspect(&self) -> &'static str {
        Aspect::IDENTIFIER
    }
    pub const fn field(&self) -> &'static str {
        Field::IDENTIFIER
    }
    pub const fn unit(&self) -> Option<&'static str> {
        Unit::NAME
    }

    pub(crate) fn binding_recipe(&self) -> ApplicationFieldBindingRecipe {
        ApplicationFieldBindingRecipe::of::<Field::Binding>(
            ApplicationFieldBindingLocus::for_relation(
                self.relation.name(),
                Aspect::IDENTIFIER,
                Field::IDENTIFIER,
            ),
        )
    }
}
