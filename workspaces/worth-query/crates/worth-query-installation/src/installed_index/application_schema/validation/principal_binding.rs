use crate::application_principal_binding::{
    WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial,
    WorthQueryPrincipalBindingInstallationDenialKind,
};

use super::super::super::WorthQueryInstalledPackageIndex;

impl WorthQueryInstalledPackageIndex {
    pub fn validate_principal_binding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        installed: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
    ) -> Result<(), WorthQueryPrincipalBindingInstallationDenial> {
        let identity = installed.binding_identity();
        if identity.runtime_ordinal() != self.runtime_ordinal() {
            return Err(principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::ForeignRuntime,
                installed,
            ));
        }
        if identity.generation() != self.generation().ordinal() {
            return Err(principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration,
                installed,
            ));
        }
        let schema = self
            .application_schemas
            .get(&(
                installed.owner().to_string(),
                installed.schema_name().to_string(),
            ))
            .ok_or_else(|| {
                principal_binding_denial(
                    WorthQueryPrincipalBindingInstallationDenialKind::SchemaMeaningChanged,
                    installed,
                )
            })?;
        let package = self.domain(installed.owner()).map_err(|_| {
            principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged,
                installed,
            )
        })?;
        if package.package_identity().digest() != identity.package_identity() {
            return Err(principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged,
                installed,
            ));
        }
        if !installed.authority_matches(&package) {
            return Err(principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch,
                installed,
            ));
        }
        let meaning_matches = schema
            .declaration()
            .members()
            .iter()
            .any(|member| installed.meaning_matches(member));
        if !meaning_matches {
            return Err(principal_binding_denial(
                WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged,
                installed,
            ));
        }
        Ok(())
    }
}

fn principal_binding_denial<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    kind: WorthQueryPrincipalBindingInstallationDenialKind,
    installed: &WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
) -> WorthQueryPrincipalBindingInstallationDenial {
    WorthQueryPrincipalBindingInstallationDenial::new(kind, installed.binding())
}
