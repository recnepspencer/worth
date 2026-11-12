//! Source and inverse roles are fixed by the actual mutation binding types.
use super::*;
use crate::application_aftermath::{ApplicationCorrectionBinding, ApplicationInverseBinding};
use crate::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationDeclaredOutputRole,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    WorthQueryRetireOutput,
};
use crate::application_schema::{
    ApplicationFieldBindingRecipe, ApplicationSchemaDeclarationDenial,
};
use std::marker::PhantomData;

pub struct ApplicationAuthoredCorrection<Schema> {
    pub(super) compose: Box<
        dyn Fn() -> Result<DeclaredAuthoredCorrection, ApplicationSchemaDeclarationDenial>
            + Send
            + Sync,
    >,
    recipe: ApplicationFieldBindingRecipe,
    pub(super) co_recipes: Vec<ApplicationFieldBindingRecipe>,
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

    /// Declare an additional independently authored Create/Retire correspondence.
    pub fn co_retire_created<C, Family, Source, Inverse>(mut self) -> Self
    where
        C: ApplicationCorrectionBinding<Schema>,
        Family: ApplicationAuthoredCorrectionFamily<Schema>,
        Source: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<Schema>>::Output,
            Action = WorthQueryCreateOutput,
        >,
        Inverse: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<
                Schema,
            >>::Output,
            Action = WorthQueryRetireOutput,
            Cardinality = Source::Cardinality,
        >,
    {
        const { <Source as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        let anchor = Self::from_family::<Family>(
            Source::NAME,
            Inverse::NAME,
            ApplicationMutationOutputPosture::Create,
        );
        let optional = <Source as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR
            .cardinality()
            == crate::application_operation::ApplicationMutationOutputRoleCardinality::AtMostOne;
        self.co_recipes.push(anchor.recipe);
        let previous = self.compose;
        self.compose =
            Box::new(move || Ok(previous()?.with_co_anchor((anchor.compose)()?, optional)));
        self
    }

    /// Restore a declared co-retired source through its own inverse Create.
    pub fn co_restore_retired<C, Family, Source, Inverse>(mut self) -> Self
    where
        C: ApplicationCorrectionBinding<Schema>,
        Family: ApplicationAuthoredCorrectionFamily<Schema>,
        Source: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<Schema>>::Output,
            Action = WorthQueryRetireOutput,
        >,
        Inverse: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<
                Schema,
            >>::Output,
            Action = WorthQueryCreateOutput,
            Cardinality = Source::Cardinality,
        >,
    {
        const { <Source as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        let anchor = Self::from_family::<Family>(
            Source::NAME,
            Inverse::NAME,
            ApplicationMutationOutputPosture::Retire,
        );
        let optional = <Source as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR
            .cardinality()
            == crate::application_operation::ApplicationMutationOutputRoleCardinality::AtMostOne;
        self.co_recipes.push(anchor.recipe);
        let previous = self.compose;
        self.compose =
            Box::new(move || Ok(previous()?.with_co_anchor((anchor.compose)()?, optional)));
        self
    }

    pub(super) fn from_family<Family: ApplicationAuthoredCorrectionFamily<Schema>>(
        source: &str,
        inverse: &str,
        posture: ApplicationMutationOutputPosture,
    ) -> Self {
        let key = Family::logical_key();
        let source_role = source.to_owned();
        let inverse_role = inverse.to_owned();
        let binding_unit = key.recipe.unit().map(|unit| unit.as_str().to_owned());
        Self {
            compose: Box::new(move || {
                Ok(DeclaredAuthoredCorrection::from_untrusted_parts(
                    DeclaredAuthoredCorrectionParts {
                        family: WorthQueryPortableTypeIdentity::declared(Family::IDENTITY),
                        logical_key: Box::new((key.resolve)()?),
                        binding_unit: binding_unit.clone(),
                        source_role: source_role.clone(),
                        inverse_role: inverse_role.clone(),
                        source_posture: posture,
                    },
                ))
            }),
            recipe: key.recipe,
            co_recipes: Vec::new(),
            marker: PhantomData,
        }
    }
    pub fn declaration(
        &self,
    ) -> Result<DeclaredAuthoredCorrection, ApplicationSchemaDeclarationDenial> {
        (self.compose)()
    }
    pub fn binding_recipes(&self) -> impl Iterator<Item = &ApplicationFieldBindingRecipe> {
        std::iter::once(&self.recipe).chain(&self.co_recipes)
    }
    pub fn binding_recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn into_declaration(
        self,
    ) -> Result<DeclaredAuthoredCorrection, ApplicationSchemaDeclarationDenial> {
        self.declaration()
    }
}
