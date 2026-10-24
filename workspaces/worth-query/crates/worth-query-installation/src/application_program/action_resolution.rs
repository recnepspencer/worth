//! Exact action occurrence and delivery resolution from real installed meaning.
use super::WorthQueryInstalledApplicationProgram;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{ApplicationAction, ApplicationActionDeclaration},
    application_schema::ApplicationSchema,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryActionResolutionDenial {
    Missing,
    Ambiguous,
    DeliveryMismatch,
    MeaningMismatch,
}
impl<Schema: ApplicationSchema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn resolve_action<Binding, Action>(
        &self,
    ) -> Result<&ApplicationActionDeclaration, WorthQueryActionResolutionDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Action: ApplicationAction<Schema, Binding>,
    {
        self.action_inventory()
            .resolve_action::<Schema, Binding, Action>()
    }
}
/// A borrowed inventory of real installed actions. It grants no execution permission.
pub struct WorthQueryInstalledActionInventory<'installed> {
    actions: &'installed [ApplicationActionDeclaration],
}
impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_> {
        WorthQueryInstalledActionInventory {
            actions: &self.actions,
        }
    }
}
impl<'installed> WorthQueryInstalledActionInventory<'installed> {
    /// Resolve a binding only when exactly one installed feature occurrence owns it.
    /// Callers retaining a declaration can subsequently resolve that complete meaning.
    pub fn resolve_unique_binding<Schema, Binding>(
        &self,
    ) -> Result<&'installed ApplicationActionDeclaration, WorthQueryActionResolutionDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut matching = self.actions.iter().filter(|action| {
            action.mutation_binding_type() == Some(std::any::TypeId::of::<Binding>())
        });
        let actual = matching
            .next()
            .ok_or(WorthQueryActionResolutionDenial::Missing)?;
        if matching.next().is_some() {
            return Err(WorthQueryActionResolutionDenial::Ambiguous);
        }
        Ok(actual)
    }

    /// Resolve selection facts, returning complete installed metadata for later comparison.
    pub fn resolve_action<Schema, Binding, Action>(
        &self,
    ) -> Result<&'installed ApplicationActionDeclaration, WorthQueryActionResolutionDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Action: ApplicationAction<Schema, Binding>,
    {
        let selector = Action::selector();
        let actual = self.resolve_occurrence::<Schema, Binding>(
            selector.composition_instance(),
            selector.feature(),
            selector.binding(),
        )?;
        if actual.mutation_binding_type() != Some(selector.mutation_binding_type())
            || actual.delivery() != Some(selector.delivery())
            || actual.required_output_source() != selector.delivery().output().is_some()
        {
            return Err(WorthQueryActionResolutionDenial::DeliveryMismatch);
        }
        Ok(actual)
    }
    /// Fresh progression compares all original installed meaning, never a partial selector.
    pub fn resolve_declared_action<Schema, Binding>(
        &self,
        declared: &ApplicationActionDeclaration,
    ) -> Result<&'installed ApplicationActionDeclaration, WorthQueryActionResolutionDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let actual = self.resolve_occurrence::<Schema, Binding>(
            declared.composition_instance(),
            declared.feature(),
            declared.binding(),
        )?;
        if actual.mutation_binding_type() != declared.mutation_binding_type() {
            return Err(WorthQueryActionResolutionDenial::DeliveryMismatch);
        }
        if actual != declared {
            return Err(WorthQueryActionResolutionDenial::MeaningMismatch);
        }
        Ok(actual)
    }
    fn resolve_occurrence<Schema, Binding>(
        &self,
        instance: &str,
        feature: &str,
        binding: &str,
    ) -> Result<&'installed ApplicationActionDeclaration, WorthQueryActionResolutionDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut matching = self.actions.iter().filter(|action| {
            action.composition_instance() == instance
                && action.feature() == feature
                && action.binding() == binding
        });
        let actual = matching
            .next()
            .ok_or(WorthQueryActionResolutionDenial::Missing)?;
        if matching.next().is_some() {
            return Err(WorthQueryActionResolutionDenial::Ambiguous);
        }
        if actual.mutation_binding_type() != Some(std::any::TypeId::of::<Binding>()) {
            return Err(WorthQueryActionResolutionDenial::DeliveryMismatch);
        }
        Ok(actual)
    }
}
