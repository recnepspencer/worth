use super::installed::WorthQueryInstalledApplicationSchema;
use super::principal_binding_match::{principal_binding_matches, principal_binding_name};
use crate::application_principal_binding::{
    WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial,
    WorthQueryPrincipalBindingInstallationDenialKind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationPrincipalBindingRef, ApplicationSchema,
};

impl<Schema> WorthQueryInstalledApplicationSchema<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn principal_binding<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        binding: ApplicationPrincipalBindingRef<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
    ) -> Result<
        WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        WorthQueryPrincipalBindingInstallationDenial,
    > {
        let installed = self
            .schema
            .members()
            .iter()
            .find(|member| principal_binding_name(member) == Some(binding.name()))
            .ok_or_else(|| {
                WorthQueryPrincipalBindingInstallationDenial::new(
                    WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled,
                    binding.name(),
                )
            })?;
        if !principal_binding_matches(installed, &binding) {
            return Err(WorthQueryPrincipalBindingInstallationDenial::new(
                WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged,
                binding.name(),
            ));
        }

        let recipe = binding.principal_identity_binding_recipe();
        let installed_value_binding = self
            .value_bindings()
            .field(
                recipe.locus().entity(),
                recipe.locus().aspect(),
                recipe.locus().field(),
            )
            .filter(|installed| {
                installed.binding_type() == recipe.binding_type()
                    && installed.value_type() == recipe.value_type()
                    && installed.identity() == recipe.binding_identity()
                    && installed.is_identity()
            })
            .ok_or_else(|| {
                WorthQueryPrincipalBindingInstallationDenial::new(
                    WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged,
                    binding.name(),
                )
            })?
            .clone();

        Ok(WorthQueryInstalledPrincipalBinding::from_installed_schema(
            self,
            binding.name(),
            binding.mapping_entity(),
            binding.identity_aspect(),
            binding.identity_field(),
            binding.status_aspect(),
            binding.status_field(),
            binding.target_relation(),
            binding.principal_entity(),
            binding.principal_identity_aspect(),
            binding.principal_identity_field(),
            binding.principal_identity_scalar_family(),
            binding.principal_identity_value_type(),
            installed_value_binding,
        ))
    }
}
