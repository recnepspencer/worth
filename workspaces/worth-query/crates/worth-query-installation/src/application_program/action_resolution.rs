//! Exact action occurrence and delivery resolution from real installed meaning.
use super::WorthQueryInstalledApplicationProgram;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryActionResolutionDenial {
    Missing,
    Ambiguous,
    DeliveryMismatch,
    MeaningMismatch,
}

impl<Schema: ApplicationSchema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn resolve_fixed_action<Binding, Action>(
        &self,
    ) -> Result<
        &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        WorthQueryActionResolutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
        Action: worth_query_declaration::facade::application_program::ApplicationFixedAction<
            Schema,
            Binding,
        >,
    {
        self.action_inventory()
            .resolve_fixed_action::<Schema, Binding, Action>()
    }
}

/// A borrowed inventory of real installed actions. It grants no execution permission.
pub struct WorthQueryInstalledActionInventory<'installed> {
    actions: &'installed [worth_query_declaration::facade::application_program::ApplicationActionDeclaration],
}
impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_> {
        WorthQueryInstalledActionInventory {
            actions: &self.actions,
        }
    }
}
impl<'installed> WorthQueryInstalledActionInventory<'installed> {
    pub fn resolve_fixed_action<Schema, Binding, Action>(
        &self,
    ) -> Result<
        &'installed worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        WorthQueryActionResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Action: worth_query_declaration::facade::application_program::ApplicationFixedAction<
            Schema,
            Binding,
        >,
    {
        let declared = Action::declaration();
        let actual = self.resolve_occurrence::<Schema, Binding>(&declared)?;
        if actual.delivery() != declared.delivery() || !actual.required_output_source() {
            return Err(WorthQueryActionResolutionDenial::DeliveryMismatch);
        }
        Ok(actual)
    }
    pub fn resolve_declared_action<Schema, Binding>(
        &self,
        declared: &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    ) -> Result<
        &'installed worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        WorthQueryActionResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let actual = self.resolve_occurrence::<Schema, Binding>(declared)?;
        if actual != declared {
            return Err(WorthQueryActionResolutionDenial::MeaningMismatch);
        }
        Ok(actual)
    }
    fn resolve_occurrence<Schema, Binding>(
        &self,
        declared: &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    ) -> Result<
        &'installed worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        WorthQueryActionResolutionDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut matching = self.actions.iter().filter(|action| {
            action.composition_instance() == declared.composition_instance()
                && action.feature() == declared.feature()
                && action.binding() == declared.binding()
        });
        let actual = matching
            .next()
            .ok_or(WorthQueryActionResolutionDenial::Missing)?;
        if matching.next().is_some() {
            return Err(WorthQueryActionResolutionDenial::Ambiguous);
        }
        if actual.mutation_binding_type() != Some(std::any::TypeId::of::<Binding>())
            || actual.mutation_binding_type() != declared.mutation_binding_type()
        {
            return Err(WorthQueryActionResolutionDenial::DeliveryMismatch);
        }
        Ok(actual)
    }
}
