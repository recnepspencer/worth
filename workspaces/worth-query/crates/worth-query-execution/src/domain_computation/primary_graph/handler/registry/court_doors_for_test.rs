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
