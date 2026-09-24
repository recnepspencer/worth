use std::sync::atomic::AtomicUsize;

use crate::admission::WorthQueryAdmittedPortableDomainPackage;
use crate::authority_cryptography::InstallationAuthorityRootKey;

use super::artifact_contract_admission::admit_artifact_contract;
use super::index_identity::{index_identity, IndexIdentityInput};
use super::*;

mod application_schema_records;

use application_schema_records::{
    compile_application_schema_records, ApplicationSchemaRecordCompilationInput,
};

#[derive(Default)]
struct InstalledIndexConstruction {
    records: BTreeMap<String, WorthQueryInstalledPackageRecord>,
    definitions:
        BTreeMap<(WorthQueryPortableDefinitionKind, String, String), WorthQueryPortableDefinition>,
    domain_operations: BTreeMap<(String, String), WorthQueryValidatedDomainOperation>,
    artifact_contracts: BTreeMap<(String, String, u32, u32), WorthQueryPortableArtifactContract>,
    artifact_contract_slots: BTreeMap<(String, u32, u32), (String, String)>,
    application_schemas: BTreeMap<
        (String, String),
        application_schema_records::PortableApplicationSchemaInstallationSeed,
    >,
    conditional_application_operations: BTreeMap<
        (String, String, String),
        WorthQueryPortableApplicationConditionalOperationBinding,
    >,
    counters: WorthQueryInstalledPackageIndexCounters,
}

impl WorthQueryInstalledPackageIndex {
    pub fn build(
        runtime: WorthQueryInstallationRuntimeIdentity,
        generation: WorthQueryInstallationGeneration,
        packages: impl IntoIterator<Item = WorthQueryAdmittedPortableDomainPackage>,
    ) -> Result<Self, WorthQueryInstalledPackageIndexDenial> {
        Self::build_with_authority_root_result(
            runtime,
            generation,
            packages,
            InstallationAuthorityRootKey::generate(),
        )
    }

    pub(super) fn build_with_authority_root_result(
        runtime: WorthQueryInstallationRuntimeIdentity,
        generation: WorthQueryInstallationGeneration,
        packages: impl IntoIterator<Item = WorthQueryAdmittedPortableDomainPackage>,
        authority_root: Result<InstallationAuthorityRootKey, ()>,
    ) -> Result<Self, WorthQueryInstalledPackageIndexDenial> {
        let authority_root = authority_root.map_err(|()| {
            WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::AuthorityEntropyUnavailable,
                "installed-package-index",
            )
        })?;
        Self::build_with_authority_root(runtime, generation, packages, authority_root)
    }

    pub(super) fn build_with_authority_root(
        runtime: WorthQueryInstallationRuntimeIdentity,
        generation: WorthQueryInstallationGeneration,
        packages: impl IntoIterator<Item = WorthQueryAdmittedPortableDomainPackage>,
        authority_root: InstallationAuthorityRootKey,
    ) -> Result<Self, WorthQueryInstalledPackageIndexDenial> {
        let mut construction = InstalledIndexConstruction::default();
        for package in packages {
            let owner = package.package().domain_identity().owner().to_string();
            construction.counters.package_rows_examined += 1;
            if construction.admit_package_owner(&owner, &package)? {
                continue;
            }
            construction.admit_package_content(&owner, &package)?;
            let authority_key = authority_root.derive_package_key(
                runtime.ordinal(),
                generation.ordinal(),
                package.package().identity().bytes(),
                package.admission_identity().bytes(),
            );
            construction.records.insert(
                owner,
                WorthQueryInstalledPackageRecord {
                    authority_key,
                    package,
                },
            );
        }
        construction.finish(runtime, generation, authority_root)
    }
}

impl InstalledIndexConstruction {
    fn finish(
        mut self,
        runtime: WorthQueryInstallationRuntimeIdentity,
        generation: WorthQueryInstallationGeneration,
        authority_root: InstallationAuthorityRootKey,
    ) -> Result<WorthQueryInstalledPackageIndex, WorthQueryInstalledPackageIndexDenial> {
        let declarations = std::mem::take(&mut self.application_schemas);
        let application_schemas =
            compile_application_schema_records(ApplicationSchemaRecordCompilationInput {
                runtime: &runtime,
                generation,
                packages: &self.records,
                declarations,
                counters: &mut self.counters,
            })?;
        self.complete_installed_counts(&application_schemas);
        let package_work = package_installation_work(&self.records);
        let application_schema_work = application_schema_installation_work(&application_schemas);
        let (identity, index_work) = index_identity(IndexIdentityInput {
            runtime: &runtime,
            generation,
            records: &self.records,
            definitions: &self.definitions,
            domain_operations: &self.domain_operations,
            artifact_contracts: &self.artifact_contracts,
            application_schemas: &application_schemas,
            conditional_application_operations: &self.conditional_application_operations,
        })
        .map_err(|denial| {
            let kind = match denial {
                worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded {
                    ..
                } => WorthQueryInstalledPackageIndexDenialKind::CanonicalEntryBudgetExceeded,
                worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
                    ..
                } => WorthQueryInstalledPackageIndexDenialKind::CanonicalEncodedByteBudgetExceeded,
                worth_foundational::facade::CanonicalDigestDerivationDenial::UnsupportedAlgorithm
                | worth_foundational::facade::CanonicalDigestDerivationDenial::RuleVersionMismatch
                | worth_foundational::facade::CanonicalDigestDerivationDenial::InputDomainMismatch
                | worth_foundational::facade::CanonicalDigestDerivationDenial::InputShapeMismatch => {
                    WorthQueryInstalledPackageIndexDenialKind::CanonicalDigestSlotRejected
                }
            };
            WorthQueryInstalledPackageIndexDenial::new(kind, "installed-index-canonical-identity")
        })?;
        Ok(WorthQueryInstalledPackageIndex {
            runtime,
            generation,
            authority_root,
            packages: self.records,
            definitions: self.definitions,
            domain_operations: self.domain_operations,
            artifact_contracts: self.artifact_contracts,
            application_schemas,
            conditional_application_operations: self.conditional_application_operations,
            identity,
            installation_canonical_work: package_work
                .combine(application_schema_work)
                .combine(index_work),
            counters: self.counters,
            indexed_operation_lookups: AtomicUsize::new(0),
        })
    }

    fn admit_package_owner(
        &mut self,
        owner: &str,
        package: &WorthQueryAdmittedPortableDomainPackage,
    ) -> Result<bool, WorthQueryInstalledPackageIndexDenial> {
        let Some(existing) = self.records.get(owner) else {
            return Ok(false);
        };
        if !existing
            .package
            .package()
            .has_same_authoritative_meaning(package.package())
        {
            return Err(WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::ConflictingPackage,
                owner,
            ));
        }
        if !existing.package.has_same_admission_authority(package) {
            return Err(WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::ConflictingAdmissionProfile,
                owner,
            ));
        }
        self.counters.equivalent_packages_converged += 1;
        Ok(true)
    }

    fn admit_package_content(
        &mut self,
        owner: &str,
        package: &WorthQueryAdmittedPortableDomainPackage,
    ) -> Result<(), WorthQueryInstalledPackageIndexDenial> {
        for definition in package.package().definitions() {
            self.counters.definition_rows_examined += 1;
            admit_definition(&mut self.definitions, owner, definition)?;
        }
        for operation in package.package().validated_domain_operations() {
            self.counters.domain_operation_rows_examined += 1;
            self.domain_operations.insert(
                (owner.to_string(), operation.definition().identity().slot()),
                operation.clone(),
            );
        }
        for contract in package.package().artifact_contracts() {
            self.counters.artifact_contract_rows_examined += 1;
            admit_artifact_contract(
                &mut self.artifact_contracts,
                &mut self.artifact_contract_slots,
                owner,
                contract,
            )?;
        }
        for schema in package.package().application_schemas() {
            self.counters.application_schema_rows_examined += 1;
            let key = (owner.to_string(), schema.name().to_string());
            if let Some(existing) = self.application_schemas.get(&key) {
                if &existing.declaration != schema {
                    return Err(WorthQueryInstalledPackageIndexDenial::new(
                        WorthQueryInstalledPackageIndexDenialKind::ConflictingApplicationSchema,
                        schema.name(),
                    ));
                }
                continue;
            }
            let spine = package.package().application_contract_spine();
            self.application_schemas.insert(
                key,
                application_schema_records::PortableApplicationSchemaInstallationSeed {
                    declaration: schema.clone(),
                    native_contracts: spine
                        .native_aspects()
                        .iter()
                        .filter(|record| record.schema() == schema.name())
                        .cloned()
                        .collect(),
                    operation_contracts: spine
                        .operations()
                        .iter()
                        .filter(|record| record.schema() == schema.name())
                        .cloned()
                        .collect(),
                },
            );
        }
        self.admit_conditional_application_operations(owner, package)?;
        Ok(())
    }

    fn admit_conditional_application_operations(
        &mut self,
        owner: &str,
        package: &WorthQueryAdmittedPortableDomainPackage,
    ) -> Result<(), WorthQueryInstalledPackageIndexDenial> {
        for binding in package.package().conditional_application_operations() {
            self.counters
                .conditional_application_operation_rows_examined += 1;
            let key = (
                owner.to_string(),
                binding.schema_name().to_string(),
                binding.application_operation().to_string(),
            );
            if let Some(existing) = self.conditional_application_operations.get(&key) {
                if existing != binding {
                    return Err(WorthQueryInstalledPackageIndexDenial::new(
                        WorthQueryInstalledPackageIndexDenialKind::ConflictingConditionalApplicationOperation,
                        binding.application_operation(),
                    ));
                }
                continue;
            }
            self.conditional_application_operations
                .insert(key, binding.clone());
        }
        Ok(())
    }

    fn complete_installed_counts(
        &mut self,
        application_schemas: &BTreeMap<
            (String, String),
            WorthQueryInstalledApplicationSchemaRecord,
        >,
    ) {
        self.counters.installed_package_count = self.records.len();
        self.counters.installed_definition_count = self.definitions.len();
        self.counters.installed_domain_operation_count = self.domain_operations.len();
        self.counters.installed_artifact_contract_count = self.artifact_contracts.len();
        self.counters.installed_application_schema_count = application_schemas.len();
        self.counters
            .installed_conditional_application_operation_count =
            self.conditional_application_operations.len();
    }
}

fn package_installation_work(
    records: &BTreeMap<String, WorthQueryInstalledPackageRecord>,
) -> WorthQueryCanonicalWorkEvidence {
    records
        .values()
        .fold(WorthQueryCanonicalWorkEvidence::zero(), |work, record| {
            work.combine(record.package.canonical_work())
        })
}

fn application_schema_installation_work(
    schemas: &BTreeMap<(String, String), WorthQueryInstalledApplicationSchemaRecord>,
) -> WorthQueryCanonicalWorkEvidence {
    schemas
        .values()
        .fold(WorthQueryCanonicalWorkEvidence::zero(), |work, record| {
            work.combine(record.installation_work())
        })
}

fn admit_definition(
    definitions: &mut BTreeMap<
        (WorthQueryPortableDefinitionKind, String, String),
        WorthQueryPortableDefinition,
    >,
    owner: &str,
    definition: &WorthQueryPortableDefinition,
) -> Result<(), WorthQueryInstalledPackageIndexDenial> {
    let key = (
        definition.kind(),
        owner.to_string(),
        definition.slot().to_string(),
    );
    if let Some(existing) = definitions.get(&key) {
        if existing == definition {
            return Ok(());
        }
        return Err(WorthQueryInstalledPackageIndexDenial::new(
            WorthQueryInstalledPackageIndexDenialKind::ConflictingDefinition,
            definition.slot(),
        ));
    }
    definitions.insert(key, definition.clone());
    Ok(())
}
