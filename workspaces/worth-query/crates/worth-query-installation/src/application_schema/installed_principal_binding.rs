use super::installed::WorthQueryInstalledApplicationSchema;
use super::principal_binding_match::{principal_binding_matches, principal_binding_name};
use crate::application_principal_binding::{
    WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial,
    WorthQueryPrincipalBindingInstallationDenialKind,
};
use crate::authority_cryptography::AuthoritySealDomain;
use worth_query_declaration::facade::application_schema::{
    ApplicationPrincipalBindingRef, ApplicationSchema, ApplicationSchemaMember,
};

mod admission_cost;
use admission_cost::{
    principal_binding_construction_work, principal_binding_copy_bytes,
    principal_meaning_comparison_work,
};

pub(crate) enum AdmittedPrincipalBindingStop<E> {
    Installation(WorthQueryPrincipalBindingInstallationDenial),
    Admission(E),
    AccountingOverflow,
}

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
        match self.principal_binding_core(
            binding,
            None::<&mut dyn FnMut(u64, u64) -> Result<(), std::convert::Infallible>>,
        ) {
            Ok(installed) => Ok(installed),
            Err(AdmittedPrincipalBindingStop::Installation(denial)) => Err(denial),
            Err(AdmittedPrincipalBindingStop::Admission(never)) => match never {},
            Err(AdmittedPrincipalBindingStop::AccountingOverflow) => {
                unreachable!("legacy binding lookup performs no accounting")
            }
        }
    }

    pub(crate) fn principal_binding_admitted<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        E,
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
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<
        WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        AdmittedPrincipalBindingStop<E>,
    > {
        self.principal_binding_core(binding, Some(prepare))
    }

    fn principal_binding_core<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        E,
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
        mut prepare: Option<&mut dyn FnMut(u64, u64) -> Result<(), E>>,
    ) -> Result<
        WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        AdmittedPrincipalBindingStop<E>,
    > {
        if let Some(admit) = prepare.as_mut() {
            let subject = u64::try_from(binding.name().len())
                .map_err(|_| AdmittedPrincipalBindingStop::AccountingOverflow)?;
            admit(
                subject
                    .checked_add(1)
                    .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?,
                subject,
            )
            .map_err(AdmittedPrincipalBindingStop::Admission)?;
        }
        let mut found = None;
        for member in self.schema.members() {
            if let Some(admit) = prepare.as_mut() {
                admit(1, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
            }
            let Some(name) = principal_binding_name(member) else {
                continue;
            };
            if let Some(admit) = prepare.as_mut() {
                let work = name
                    .len()
                    .checked_add(binding.name().len())
                    .and_then(|n| n.checked_add(1))
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?;
                admit(work, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
            }
            if name == binding.name() {
                found = Some(member);
                break;
            }
        }
        let installed = found.ok_or_else(|| {
            AdmittedPrincipalBindingStop::Installation(
                WorthQueryPrincipalBindingInstallationDenial::new(
                    WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled,
                    binding.name(),
                ),
            )
        })?;
        if let Some(admit) = prepare.as_mut() {
            admit(12, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
            let work = principal_meaning_comparison_work(installed, &binding)
                .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?;
            admit(work, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
        }
        if !principal_binding_matches(installed, &binding) {
            return Err(AdmittedPrincipalBindingStop::Installation(
                WorthQueryPrincipalBindingInstallationDenial::new(
                    WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged,
                    binding.name(),
                ),
            ));
        }
        let recipe = binding.principal_identity_binding_recipe();
        let located = if let Some(admit) = prepare.as_mut() {
            self.value_bindings()
                .field_admitted(
                    recipe.locus().entity(),
                    recipe.locus().aspect(),
                    recipe.locus().field(),
                    admit,
                )
                .map_err(|stop| match stop {
                    super::value_binding::AdmittedFieldBindingStop::Admission(stop) => {
                        AdmittedPrincipalBindingStop::Admission(stop)
                    }
                    super::value_binding::AdmittedFieldBindingStop::AccountingOverflow => {
                        AdmittedPrincipalBindingStop::AccountingOverflow
                    }
                })?
        } else {
            self.value_bindings().field(
                recipe.locus().entity(),
                recipe.locus().aspect(),
                recipe.locus().field(),
            )
        };
        if let (Some(admit), Some(installed)) = (prepare.as_mut(), located) {
            let work = installed
                .identity()
                .as_str()
                .len()
                .checked_add(recipe.binding_identity().as_str().len())
                .and_then(|n| n.checked_add(4))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?;
            admit(work, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
        }
        let installed_value_binding = located
            .filter(|installed| {
                installed.binding_type() == recipe.binding_type()
                    && installed.value_type() == recipe.value_type()
                    && installed.identity() == recipe.binding_identity()
                    && installed.is_identity()
            })
            .ok_or_else(|| {
                AdmittedPrincipalBindingStop::Installation(
                    WorthQueryPrincipalBindingInstallationDenial::new(
                        WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged,
                        binding.name(),
                    ),
                )
            })?;
        if let Some(admit) = prepare.as_mut() {
            // Sixteen cloned text widths and fourteen authority transcript
            // widths are measured before their variable copy/hash claim.
            admit(30, 0).map_err(AdmittedPrincipalBindingStop::Admission)?;
            let copy = principal_binding_copy_bytes(self, &binding, installed_value_binding)
                .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?;
            let work = principal_binding_construction_work(self, &binding, copy)
                .ok_or(AdmittedPrincipalBindingStop::AccountingOverflow)?;
            admit(work, copy).map_err(AdmittedPrincipalBindingStop::Admission)?;
        }
        let installed_value_binding = installed_value_binding.clone();
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
