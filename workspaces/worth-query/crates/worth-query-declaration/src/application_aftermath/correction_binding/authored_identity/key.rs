//! A typed family key carries the existing native field recipe and portable meaning.
use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationFieldBindingRecipe, ApplicationFieldMarkerIdentity, ApplicationFieldRef,
    ApplicationFieldUnit, ApplicationSchemaMember,
};
use std::marker::PhantomData;

pub struct ApplicationAuthoredLogicalKey<Schema, Entity> {
    pub(super) member: ApplicationSchemaMember,
    pub(super) recipe: ApplicationFieldBindingRecipe,
    marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity: ApplicationEntityMarkerIdentity<Schema>>
    ApplicationAuthoredLogicalKey<Schema, Entity>
{
    pub fn from_field<Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
        Field: ApplicationFieldMarkerIdentity<Schema, Entity, Aspect, Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        let recipe = field.binding_recipe();
        Self {
            member: ApplicationSchemaMember::Field {
                entity: Entity::IDENTIFIER.to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
                presence: Field::PRESENCE,
                scalar_family: recipe.scalar_family(),
                value_type: recipe.binding_identity().as_str().to_owned(),
                canonical: recipe.canonical_contract().cloned(),
                unit: field.unit().map(str::to_owned),
                frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
                writable: false,
                equality_queryable: false,
                unique: false,
            },
            recipe,
            marker: PhantomData,
        }
    }
}
