//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> PendingMutationHandlerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn seal_for_test(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<InstalledMutationHandlerRegistry<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    {
        Self::seal(self, installed_schema, binding_identity)
    }
}

impl<Schema> InstalledMutationHandlerRegistry<Schema> {
    #[doc(hidden)]
    pub fn entries_for_test(&self) -> &BTreeMap<String, PendingMutationHandler> {
        &self.entries
    }
}

impl<Schema> InstalledMutationHandlerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn get_for_test<Binding>(&self) -> Option<Arc<dyn OperationHandler<Schema, Binding>>>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        Self::get::<Binding>(self)
    }
}

impl PendingMutationHandler {
    #[doc(hidden)]
    pub fn value_mut_for_test(&mut self) -> &mut Arc<dyn Any + Send + Sync> {
        &mut self.value
    }
}

impl<Schema> InstalledMutationHandlerRegistry<Schema> {
    #[doc(hidden)]
    pub fn entries_mut_for_test(&mut self) -> &mut BTreeMap<String, PendingMutationHandler> {
        &mut self.entries
    }
}
