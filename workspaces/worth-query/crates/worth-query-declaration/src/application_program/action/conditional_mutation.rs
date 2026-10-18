//! Conditional membership is distinct from ordinary mutation-call authority.
use super::{
    ApplicationActionDeclaration, ApplicationActionInstanceRef, ApplicationActionShape,
    ApplicationCompositionInstance, ApplicationFeature, ApplicationMutationBinding,
    ApplicationSchema,
};

impl ApplicationActionDeclaration {
    pub(crate) fn conditional_mutation_identity(&self) -> Option<&'static str> {
        (self.conditional_only && self.mutation_binding_type.is_some()).then_some(self.binding)
    }

    pub(in crate::application_program) fn conditional_mutation<Schema, Instance, Feature, Binding>(
    ) -> Self
    where
        Schema: ApplicationSchema,
        Instance: ApplicationCompositionInstance,
        Feature: ApplicationFeature<Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut action =
            ApplicationActionInstanceRef::<Schema, Instance, Feature, Binding>::declaration();
        action.conditional_only = true;
        action
    }
}
