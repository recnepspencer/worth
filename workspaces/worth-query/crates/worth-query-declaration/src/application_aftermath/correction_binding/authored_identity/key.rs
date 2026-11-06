//! A typed family key carries the existing native field recipe and portable meaning.
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEntityMarkerIdentity, ApplicationFieldBindingRecipe,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};
use std::marker::PhantomData;

pub struct ApplicationAuthoredLogicalKey<Schema, Entity> {
    pub(super) resolve: Box<
        dyn Fn() -> Result<ApplicationSchemaMember, ApplicationSchemaDeclarationDenial>
            + Send
            + Sync,
    >,
    pub(super) recipe: ApplicationFieldBindingRecipe,
    marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity: ApplicationEntityMarkerIdentity<Schema>>
    ApplicationAuthoredLogicalKey<Schema, Entity>
{
    /// Records the selection without resolving it; operation association checks its metadata.
    pub fn from_field<Field: RecordSelection<Entity = Entity>>(field: Field) -> Self {
        Self {
            resolve: Box::new(move || {
                let checked = field.checked()?;
                let recipe = field.expected_recipe();
                Ok(ApplicationSchemaMember::Field {
                    entity: Entity::IDENTIFIER.to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                    presence: checked.presence(),
                    scalar_family: recipe.scalar_family(),
                    value_type: recipe.binding_identity().as_str().to_owned(),
                    canonical: recipe.canonical_contract().cloned(),
                    content_format: checked
                        .content_format()
                        .map(|binding| binding.declaration().identity),
                    unit: field.unit().map(str::to_owned),
                    frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
                    writable: false,
                    equality_queryable: false,
                    unique: false,
                })
            }),
            recipe: field.expected_recipe(),
            marker: PhantomData,
        }
    }
}
