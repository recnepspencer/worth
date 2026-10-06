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

#[derive(Debug)]
pub enum WorthQueryRetainedMutationBindingAdmissionStop<Stop> {
    Admission(Stop),
    Installation(InstallationDenial),
    AccountingOverflow,
}

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

fn ordered_lookup_work(entries: usize, selected_text_bytes: usize) -> Option<u64> {
    let levels = usize::BITS.checked_sub(entries.leading_zeros())?;
    u64::from(levels)
        .checked_mul(11)?
        .checked_mul(u64::try_from(selected_text_bytes.checked_add(2)?).ok()?)?
        .checked_add(1)
}

impl WorthQueryInstalledPackageIndex {
    /// Admit the selected installation lookups, comparisons and possible denial
    /// copy before issuing a borrowed proof. The ordinary public validators are
    /// unchanged. Cold producer setup already compiled and ownership-checked
    /// this opaque binding from `schema`; matching its catalog Arc authenticates
    /// that issuer without recompiling the immutable operation contract.
    pub fn validate_retained_mutation_binding_admitted<'a, Schema, Binding, Stop>(
        &'a self,
        schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
        binding: &'a WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        WorthQueryCurrentRetainedMutationBinding<'a, Schema, Binding>,
        WorthQueryRetainedMutationBindingAdmissionStop<Stop>,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        use WorthQueryRetainedMutationBindingAdmissionStop as StopKind;

        prepare(12, 0).map_err(StopKind::Admission)?;
        let operation = binding.operation();
        let subject = operation.operation();
        let subject_bytes =
            u64::try_from(subject.len()).map_err(|_| StopKind::AccountingOverflow)?;
        prepare(
            subject_bytes
                .checked_add(1)
                .ok_or(StopKind::AccountingOverflow)?,
            subject_bytes,
        )
        .map_err(StopKind::Admission)?;
        let denied = |kind| StopKind::Installation(InstallationDenial::new(kind, subject));

        let issuer = operation.binding_identity();
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
        let name = schema.schema_name();
        let key_bytes = owner
            .len()
            .checked_add(name.len())
            .ok_or(StopKind::AccountingOverflow)?;
        let key_work = ordered_lookup_work(self.application_schemas.len(), key_bytes)
            .ok_or(StopKind::AccountingOverflow)?;
        let key_bytes = u64::try_from(key_bytes).map_err(|_| StopKind::AccountingOverflow)?;
        prepare(
            key_work
                .checked_add(key_bytes)
                .ok_or(StopKind::AccountingOverflow)?,
            key_bytes,
        )
        .map_err(StopKind::Admission)?;
        let current = self
            .application_schemas
            .get(&(owner.to_owned(), name.to_owned()))
            .ok_or_else(|| denied(DenialKind::SchemaMeaningChanged))?;

        // This package identity embeds the full canonical schema basis: its
        // header, member sequence and contribution provenance. The retained
        // schema was originally bound from that installed package, and its
        // original mutation catalog holds the exact compiled binding Arc.
        // The getter returns one copied 32-byte digest before its 32-byte
        // comparison with the retained schema identity.
        prepare(66, 0).map_err(StopKind::Admission)?;
        if current.schema_identity() != schema.schema_identity {
            return Err(denied(DenialKind::SchemaMeaningChanged));
        }

        let package_work = ordered_lookup_work(self.packages.len(), owner.len())
            .ok_or(StopKind::AccountingOverflow)?;
        prepare(package_work, 0).map_err(StopKind::Admission)?;
        let package = self
            .packages
            .get(owner)
            .ok_or_else(|| denied(DenialKind::PackageIdentityChanged))?;
        // Package, admission, authority-key and binding-package identities
        // each compare one initialized 32-byte value, plus fixed owner visits.
        prepare(136, 0).map_err(StopKind::Admission)?;
        if package.package.package().identity() != schema.package_authority.package_identity()
            || package.package.admission_identity() != schema.package_authority.admission_identity()
            || !package
                .authority_key
                .matches(&schema.package_authority.authority_key)
            || issuer.package_identity() != schema.package_authority.package_identity().digest()
        {
            return Err(denied(DenialKind::PackageIdentityChanged));
        }

        let catalog_work = ordered_lookup_work(
            schema.mutation_catalog.binding_count(),
            Binding::IDENTITY.len(),
        )
        .ok_or(StopKind::AccountingOverflow)?;
        let issuer_text = owner
            .len()
            .checked_add(name.len())
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(StopKind::AccountingOverflow)?;
        // Constructing one issuer identity initializes 80 bytes; comparison
        // against both the operation and principal visits two more 80-byte
        // identities. `issuer_text` funds both initialized operands of the
        // four owner/schema text comparisons; fixed visits cover headers/Arc.
        prepare(
            catalog_work
                .checked_add(248)
                .and_then(|n| n.checked_add(issuer_text))
                .ok_or(StopKind::AccountingOverflow)?,
            0,
        )
        .map_err(StopKind::Admission)?;
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
