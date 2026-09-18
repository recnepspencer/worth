use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{ApplicationChangeShape, ApplicationLocalityScope},
    application_schema::ApplicationSchema,
};

use super::WorthQueryInstalledApplicationProgram;

/// Installed evidence that one mutation uses an exact locality and change contract.
pub struct WorthQueryInstalledScopedAction<Schema, Binding, Scope, Shape> {
    marker: PhantomData<fn() -> (Schema, Binding, Scope, Shape)>,
}

impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn action_for_mutation<Binding>(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_program::ApplicationActionDeclaration>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let binding = std::any::TypeId::of::<Binding>();
        self.actions
            .iter()
            .find(|action| action.mutation_binding_type() == Some(binding))
    }

    pub fn scoped_action<Binding, Scope, Shape>(
        &self,
    ) -> Option<WorthQueryInstalledScopedAction<Schema, Binding, Scope, Shape>>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        let binding = std::any::TypeId::of::<Binding>();
        let scope = std::any::TypeId::of::<Scope>();
        let shape = std::any::TypeId::of::<Shape>();
        self.actions
            .iter()
            .any(|action| {
                action.mutation_binding_type() == Some(binding)
                    && action
                        .locality()
                        .is_some_and(|declared| declared.scope_type() == scope)
                    && action
                        .change_shape()
                        .is_some_and(|declared| declared.shape_type() == shape)
            })
            .then_some(WorthQueryInstalledScopedAction {
                marker: PhantomData,
            })
    }
}
