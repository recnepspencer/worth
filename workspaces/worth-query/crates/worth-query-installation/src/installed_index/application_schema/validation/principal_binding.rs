use crate::application_principal_binding::{
    WorthQueryInstalledPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial,
    WorthQueryPrincipalBindingInstallationDenialKind,
};
use crate::authority_cryptography::InstallationAuthorityLineage;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use super::super::super::WorthQueryInstalledPackageIndex;

/// Cold validation of one exact principal contract against one immutable
/// installed package index. A later selected read still resolves the external
/// principal and its native mapping at the current Product.
pub struct WorthQueryValidatedPrincipalBinding {
    index_identity: super::super::super::WorthQueryInstalledPackageIndexIdentity,
    authority_lineage: InstallationAuthorityLineage,
    schema_identity:
        worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
    binding: String,
    authority_identity: String,
    runtime_ordinal: u64,
    generation: u64,
}

impl WorthQueryValidatedPrincipalBinding {
    pub fn comparison_work_bound(&self, binding: &str) -> Option<u64> {
        self.binding.len().checked_add(binding.len())
            .and_then(|n| n.checked_add(self.authority_identity.len().checked_mul(2)?))
            .and_then(|n| n.checked_add(4 * 32 + 2 * std::mem::size_of::<worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity>() + 4 * std::mem::size_of::<u64>() + 8))
            .and_then(|n| u64::try_from(n).ok())
    }
}

#[derive(Debug)]
pub enum WorthQueryPrincipalBindingValidationAdmissionStop<Stop> {
    Admission(Stop),
    Validation(WorthQueryPrincipalBindingInstallationDenial),
    AccountingOverflow,
}

fn ordered_lookup_work(entries: usize, text_bytes: usize) -> Option<u64> {
    let levels = usize::BITS.checked_sub(entries.leading_zeros())?;
    u64::from(levels)
        .checked_mul(11)?
        .checked_mul(u64::try_from(text_bytes.checked_add(2)?).ok()?)?
        .checked_add(1)
}

impl WorthQueryInstalledPackageIndex {
    pub fn retain_validated_principal_binding<
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
    ) -> Result<WorthQueryValidatedPrincipalBinding, WorthQueryPrincipalBindingInstallationDenial>
    {
        self.validate_principal_binding(installed)?;
        Ok(WorthQueryValidatedPrincipalBinding {
            index_identity: self.identity().clone(),
            authority_lineage: self.authority_lineage,
            schema_identity: installed.binding_identity().clone(),
            binding: installed.binding().to_owned(),
            authority_identity: installed.authority_identity().to_owned(),
            runtime_ordinal: self.runtime_ordinal(),
            generation: self.generation().ordinal(),
        })
    }

    pub fn retains_validated_principal_binding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        retained: &WorthQueryValidatedPrincipalBinding,
        installed: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
    ) -> bool {
        &retained.index_identity == self.identity()
            && retained.authority_lineage == self.authority_lineage
            && &retained.schema_identity == installed.binding_identity()
            && retained.binding == installed.binding()
            && retained.authority_identity == installed.authority_identity()
            && retained.runtime_ordinal == self.runtime_ordinal()
            && retained.generation == self.generation().ordinal()
    }
    /// Prepare the exact immutable installation reads and owned copies before
    /// invoking the existing principal-binding validator. The cost walk has
    /// no authority; the unchanged validator remains the sole acceptance path.
    pub fn validate_principal_binding_admitted<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Stop,
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
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryPrincipalBindingValidationAdmissionStop<Stop>> {
        use WorthQueryPrincipalBindingValidationAdmissionStop as Denied;
        let binding_bytes =
            u64::try_from(installed.binding().len()).map_err(|_| Denied::AccountingOverflow)?;
        prepare(binding_bytes.saturating_add(2), binding_bytes).map_err(Denied::Admission)?;
        if installed.binding_identity().runtime_ordinal() != self.runtime_ordinal()
            || installed.binding_identity().generation() != self.generation().ordinal()
        {
            return self
                .validate_principal_binding(installed)
                .map_err(Denied::Validation);
        }

        let schema_text = installed
            .owner()
            .len()
            .checked_add(installed.schema_name().len())
            .ok_or(Denied::AccountingOverflow)?;
        let schema_lookup = ordered_lookup_work(self.application_schemas.len(), schema_text)
            .ok_or(Denied::AccountingOverflow)?;
        let schema_text = u64::try_from(schema_text).map_err(|_| Denied::AccountingOverflow)?;
        // First search obtains the member inventory; validation performs a
        // second search with its own freshly constructed owned key.
        prepare(
            schema_lookup
                .checked_add(schema_text)
                .ok_or(Denied::AccountingOverflow)?,
            schema_text,
        )
        .map_err(Denied::Admission)?;
        let schema = self.application_schemas.get(&(
            installed.owner().to_owned(),
            installed.schema_name().to_owned(),
        ));
        // The validator performs the same lookup with a new owned key.
        prepare(
            schema_lookup
                .checked_add(schema_text)
                .ok_or(Denied::AccountingOverflow)?,
            schema_text,
        )
        .map_err(Denied::Admission)?;
        let package_lookup = ordered_lookup_work(self.packages.len(), installed.owner().len())
            .ok_or(Denied::AccountingOverflow)?;
        let owner_bytes =
            u64::try_from(installed.owner().len()).map_err(|_| Denied::AccountingOverflow)?;
        prepare(
            package_lookup
                .checked_add(
                    owner_bytes
                        .checked_mul(2)
                        .ok_or(Denied::AccountingOverflow)?,
                )
                .ok_or(Denied::AccountingOverflow)?,
            owner_bytes
                .checked_mul(2)
                .ok_or(Denied::AccountingOverflow)?,
        )
        .map_err(Denied::Admission)?;

        prepare(14, 0).map_err(Denied::Admission)?;
        let transcript_work = installed
            .authority_validation_work_bound()
            .ok_or(Denied::AccountingOverflow)?;
        prepare(transcript_work, 0).map_err(Denied::Admission)?;
        if let Some(schema) = schema {
            for member in schema.declaration().members() {
                prepare(1, 0).map_err(Denied::Admission)?;
                if matches!(member, ApplicationSchemaMember::PrincipalBinding { .. }) {
                    prepare(11, 0).map_err(Denied::Admission)?;
                }
                let comparison = installed
                    .meaning_comparison_work_bound(member)
                    .ok_or(Denied::AccountingOverflow)?;
                prepare(comparison, 0).map_err(Denied::Admission)?;
            }
        }
        self.validate_principal_binding(installed)
            .map_err(Denied::Validation)
    }

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
