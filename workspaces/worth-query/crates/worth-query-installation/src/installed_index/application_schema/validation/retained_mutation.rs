//! Current issuer proof for a mutation binding compiled at producer installation.

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

use crate::application_operation::{
    WorthQueryApplicationOperationInstallationDenial as InstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind as DenialKind,
    WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledBoundMutationOperation,
    WorthQueryInstalledBoundMutationPrincipal,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;

use super::super::super::WorthQueryInstalledPackageIndex;

/// An exact installed issuer and its cold-compiled, typed mutation binding.
/// The index, schema and binding are borrowed together so a caller cannot move
/// the result to another installed runtime or generation.
pub struct WorthQueryCurrentRetainedMutationBinding<'a, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    binding: &'a WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
    _schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    _index: &'a WorthQueryInstalledPackageIndex,
}

impl<'a, Schema, Binding> WorthQueryCurrentRetainedMutationBinding<'a, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// The typed operation may be used only with the runtime installation
    /// that issued this proof. A later equivalent index rebuild can issue a
    /// fresh proof; it cannot be substituted into an existing proof.
    pub fn belongs_to(
        &self,
        index: &WorthQueryInstalledPackageIndex,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> bool {
        std::ptr::eq(self._index, index) && std::ptr::eq(self._schema, schema)
    }

    pub fn operation(&self) -> &WorthQueryInstalledBoundMutationOperation<Schema, Binding> {
        self.binding.operation()
    }

    pub fn principal_binding(&self) -> &WorthQueryInstalledBoundMutationPrincipal<Schema, Binding> {
        self.binding.principal_binding()
    }

    pub fn scope_contract(
        &self,
    ) -> &worth_query_declaration::facade::application_operation::ApplicationMutationScopeContract
    {
        self.binding.scope_contract()
    }
}

impl WorthQueryInstalledPackageIndex {
    /// Issue a borrowed binding only from the current native installation catalog.
    pub fn validate_retained_mutation_binding<'a, Schema, Binding>(
        &'a self,
        schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
        binding: &'a WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
    ) -> Result<WorthQueryCurrentRetainedMutationBinding<'a, Schema, Binding>, InstallationDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let issuer = binding.operation().binding_identity();
        let denied = |kind| InstallationDenial::new(kind, binding.operation().operation());
        if issuer.runtime_ordinal() != self.runtime_ordinal()
            || schema.package_authority.runtime_ordinal != self.runtime_ordinal()
        {
            return Err(denied(DenialKind::ForeignRuntime));
        }
        if issuer.generation() != self.generation().ordinal()
            || schema.package_authority.generation.ordinal() != self.generation().ordinal()
        {
            return Err(denied(DenialKind::StaleGeneration));
        }
        let owner = schema.owner();
        let current = self
            .application_schemas
            .get(&(owner.to_owned(), schema.schema_name().to_owned()))
            .ok_or_else(|| denied(DenialKind::SchemaMeaningChanged))?;
        if current.schema_identity() != schema.schema_identity {
            return Err(denied(DenialKind::SchemaMeaningChanged));
        }
        let package = self
            .packages
            .get(owner)
            .ok_or_else(|| denied(DenialKind::PackageIdentityChanged))?;
        if package.package.package().identity() != schema.package_authority.package_identity()
            || package.package.admission_identity() != schema.package_authority.admission_identity()
            || !package
                .authority_key
                .matches(&schema.package_authority.authority_key)
            || issuer.package_identity() != schema.package_authority.package_identity().digest()
        {
            return Err(denied(DenialKind::PackageIdentityChanged));
        }
        if !binding.matches_issuing_schema(schema) {
            return Err(denied(DenialKind::MutationBindingMeaningChanged));
        }
        Ok(WorthQueryCurrentRetainedMutationBinding {
            binding,
            _schema: schema,
            _index: self,
        })
    }
}
