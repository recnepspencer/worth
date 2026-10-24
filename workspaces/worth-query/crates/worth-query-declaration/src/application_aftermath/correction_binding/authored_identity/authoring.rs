//! Source and inverse roles are fixed by the actual mutation binding types.
use super::*;
use crate::application_aftermath::{ApplicationCorrectionBinding, ApplicationInverseBinding};
use crate::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationDeclaredOutputRole,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    WorthQueryRetireOutput,
};
use crate::application_schema::ApplicationFieldBindingRecipe;
use std::marker::PhantomData;

pub struct ApplicationAuthoredCorrection<Schema> {
    declaration: DeclaredAuthoredCorrection,
    recipe: ApplicationFieldBindingRecipe,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema: ApplicationSchema> ApplicationAuthoredCorrection<Schema> {
    pub fn retire_created<C, Family, Source, Inverse>() -> Self
    where
        C: ApplicationCorrectionBinding<Schema>,
        Family: ApplicationAuthoredCorrectionFamily<Schema>,
        Source: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<Schema>>::Output,
            Action = WorthQueryCreateOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Inverse: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<
                Schema,
            >>::Output,
            Action = WorthQueryRetireOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
    {
        const { <Source as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        Self::from_family::<Family>(
            Source::NAME,
            Inverse::NAME,
            ApplicationMutationOutputPosture::Create,
        )
    }

    pub fn restore_retired<C, Family, Source, Inverse>() -> Self
    where
        C: ApplicationCorrectionBinding<Schema>,
        Family: ApplicationAuthoredCorrectionFamily<Schema>,
        Source: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<Schema>>::Output,
            Action = WorthQueryRetireOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Inverse: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<
                Schema,
            >>::Output,
            Action = WorthQueryCreateOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
    {
        const { <Source as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        Self::from_family::<Family>(
            Source::NAME,
            Inverse::NAME,
            ApplicationMutationOutputPosture::Retire,
        )
    }

    fn from_family<Family: ApplicationAuthoredCorrectionFamily<Schema>>(
        source: &str,
        inverse: &str,
        posture: ApplicationMutationOutputPosture,
    ) -> Self {
        let key = Family::logical_key();
        Self {
            declaration: DeclaredAuthoredCorrection::from_untrusted_parts(
                DeclaredAuthoredCorrectionParts {
                    family: WorthQueryPortableTypeIdentity::declared(Family::IDENTITY),
                    logical_key: Box::new(key.member),
                    binding_unit: key.recipe.unit().map(|unit| unit.as_str().to_owned()),
                    source_role: source.to_owned(),
                    inverse_role: inverse.to_owned(),
                    source_posture: posture,
                },
            ),
            recipe: key.recipe,
            marker: PhantomData,
        }
    }
    pub fn declaration(&self) -> &DeclaredAuthoredCorrection {
        &self.declaration
    }
    pub fn binding_recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn into_declaration(self) -> DeclaredAuthoredCorrection {
        self.declaration
    }
}
