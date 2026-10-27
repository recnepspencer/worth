use super::{
    catalog::CompiledApplicationResource,
    WorthQueryApplicationResourceInstallationDenial as Denial,
    WorthQueryApplicationResourceInstallationDenialKind as Kind,
};
use crate::{
    application_operation::WorthQueryInstalledAbilityRequirement,
    application_query::WorthQueryInstalledApplicationQueryBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
};
use std::{marker::PhantomData, sync::Arc};
use worth_query_declaration::facade::{
    application_resource::{
        ApplicationResourceAction, ApplicationResourceBinding, ErasedApplicationResourceDefinition,
    },
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};

/// Installed declaration/recipe correspondence. This contains no provider
/// installation, request custody, action permission or completion authority.
pub struct WorthQueryInstalledApplicationResource<
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
> {
    compiled: Arc<CompiledApplicationResource>,
    access: WorthQueryInstalledApplicationQueryBinding<Schema, Binding::Access>,
    _binding: PhantomData<fn() -> Binding>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>
    WorthQueryInstalledApplicationResource<Schema, Binding>
{
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.compiled.schema
    }
    pub fn definition(&self) -> &ErasedApplicationResourceDefinition {
        &self.compiled.definition
    }
    pub fn access_binding(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryBinding<Schema, Binding::Access> {
        &self.access
    }
    pub fn action_requirement(
        &self,
        action: ApplicationResourceAction,
    ) -> Option<&WorthQueryInstalledAbilityRequirement> {
        self.compiled.requirements.get(&action)
    }
    pub fn access_contract(
        &self,
        purpose: super::WorthQueryResourceAccessPurpose,
    ) -> &super::WorthQueryInstalledResourceAccessContract {
        &self.compiled.access_contracts[&purpose]
    }
    #[doc(hidden)]
    pub fn retain_access_contract(
        &self,
        purpose: super::WorthQueryResourceAccessPurpose,
    ) -> Arc<super::WorthQueryInstalledResourceAccessContract> {
        Arc::clone(&self.compiled.access_contracts[&purpose])
    }
    #[doc(hidden)]
    pub fn owns_access_contract(
        &self,
        contract: &super::WorthQueryInstalledResourceAccessContract,
    ) -> bool {
        std::ptr::eq(self.access_contract(contract.purpose()), contract)
    }
}

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    pub fn installed_resource<Binding: ApplicationResourceBinding<Schema>>(
        &self,
    ) -> Result<WorthQueryInstalledApplicationResource<Schema, Binding>, Denial> {
        let compiled = self
            .resource_catalog
            .get(Binding::IDENTITY)
            .ok_or_else(|| Denial::new(Kind::ResourceNotInstalled, Binding::IDENTITY))?;
        if !compiled.descriptor.matches_binding::<Schema, Binding>() {
            return Err(Denial::new(Kind::BindingMeaningChanged, Binding::IDENTITY));
        }
        let access = self
            .installed_query_binding::<Binding::Access>()
            .map_err(|cause| Denial::access(Binding::IDENTITY, cause))?;
        if access.query().identity() != compiled.access.query_identity() {
            return Err(Denial::new(Kind::AccessBindingMismatch, Binding::IDENTITY));
        }
        Ok(WorthQueryInstalledApplicationResource {
            compiled,
            access,
            _binding: PhantomData,
        })
    }
}
