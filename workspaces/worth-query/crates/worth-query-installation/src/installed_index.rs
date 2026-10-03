use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

mod application_schema;
mod application_schema_record;
mod artifact_contract_admission;
mod artifact_contract_authority;
mod authority;
mod authority_validation;
mod conditional_application_operation;
mod construction;
mod denial;
mod index_identity;
mod rebuild_report;
mod relation;

pub use application_schema::{
    WorthQueryCurrentRetainedMutationBinding, WorthQueryRetainedMutationBindingAdmissionStop,
};
pub use application_schema::{
    WorthQueryPrincipalBindingValidationAdmissionStop, WorthQueryValidatedPrincipalBinding,
};
pub use authority::WorthQueryInstalledPackageAuthority;
pub use denial::{
    WorthQueryInstalledPackageIndexDenial, WorthQueryInstalledPackageIndexDenialKind,
};
pub use index_identity::WorthQueryInstalledPackageIndexIdentity;
pub use rebuild_report::{
    WorthQueryInstalledPackageIndexCounters, WorthQueryInstalledPackageIndexRebuildReport,
};
pub use relation::WorthQueryInstalledPackageIndexRelation;

use crate::admission::WorthQueryAdmittedPortableDomainPackage;
use crate::application_operation::WorthQueryPortableApplicationConditionalOperationBinding;
use crate::authority_cryptography::InstallationAuthorityLineage;
use crate::authority_cryptography::{InstallationAuthorityRootKey, PackageAuthorityKey};
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::domain_computation::WorthQueryPortableArtifactContract;
use crate::domain_operation::WorthQueryValidatedDomainOperation;
use crate::generation::{WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity};
use crate::installed_domain_operation::WorthQueryInstalledDomainOperationAuthority;
use crate::installed_operation::WorthQueryInstalledOperationAuthority;
use crate::package::{WorthQueryPortableDefinition, WorthQueryPortableDefinitionKind};
use application_schema_record::WorthQueryInstalledApplicationSchemaRecord;

#[derive(Debug)]
struct WorthQueryInstalledPackageRecord {
    package: WorthQueryAdmittedPortableDomainPackage,
    authority_key: PackageAuthorityKey,
}

#[derive(Debug)]
pub struct WorthQueryInstalledPackageIndex {
    runtime: WorthQueryInstallationRuntimeIdentity,
    generation: WorthQueryInstallationGeneration,
    authority_root: InstallationAuthorityRootKey,
    authority_lineage: InstallationAuthorityLineage,
    packages: BTreeMap<String, WorthQueryInstalledPackageRecord>,
    definitions:
        BTreeMap<(WorthQueryPortableDefinitionKind, String, String), WorthQueryPortableDefinition>,
    domain_operations: BTreeMap<(String, String), WorthQueryValidatedDomainOperation>,
    artifact_contracts: BTreeMap<(String, String, u32, u32), WorthQueryPortableArtifactContract>,
    application_schemas: BTreeMap<(String, String), WorthQueryInstalledApplicationSchemaRecord>,
    conditional_application_operations: BTreeMap<
        (String, String, String),
        WorthQueryPortableApplicationConditionalOperationBinding,
    >,
    identity: WorthQueryInstalledPackageIndexIdentity,
    installation_canonical_work: WorthQueryCanonicalWorkEvidence,
    counters: WorthQueryInstalledPackageIndexCounters,
    indexed_operation_lookups: AtomicUsize,
}

impl WorthQueryInstalledPackageIndex {
    pub fn installation_runtime(&self) -> &WorthQueryInstallationRuntimeIdentity {
        &self.runtime
    }

    pub fn runtime_ordinal(&self) -> u64 {
        self.runtime.ordinal()
    }

    pub fn generation(&self) -> WorthQueryInstallationGeneration {
        self.generation
    }

    pub fn identity(&self) -> &WorthQueryInstalledPackageIndexIdentity {
        &self.identity
    }

    pub const fn installation_canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.installation_canonical_work
    }

    pub fn counters(&self) -> WorthQueryInstalledPackageIndexCounters {
        self.counters
    }

    pub fn installed_definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn installed_domain_operation_count(&self) -> usize {
        self.domain_operations.len()
    }

    pub fn installed_artifact_contract_count(&self) -> usize {
        self.artifact_contracts.len()
    }

    pub fn installed_application_schema_count(&self) -> usize {
        self.application_schemas.len()
    }

    pub fn installed_conditional_application_operation_count(&self) -> usize {
        self.conditional_application_operations.len()
    }

    pub fn indexed_operation_lookups(&self) -> usize {
        self.indexed_operation_lookups.load(Ordering::Relaxed)
    }

    pub fn domain(
        &self,
        owner: &str,
    ) -> Result<WorthQueryInstalledPackageAuthority, WorthQueryInstalledPackageIndexDenial> {
        let record = self.packages.get(owner).ok_or_else(|| {
            WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::DomainNotInstalled,
                owner,
            )
        })?;
        Ok(WorthQueryInstalledPackageAuthority {
            runtime_ordinal: self.runtime.ordinal(),
            generation: self.generation,
            owner: owner.to_string(),
            package_identity: record.package.package().identity().clone(),
            admission_identity: record.package.admission_identity().clone(),
            authority_key: record.authority_key.clone(),
        })
    }

    pub fn operation(
        &self,
        owner: &str,
        operation_slot: &str,
    ) -> Result<WorthQueryInstalledOperationAuthority, WorthQueryInstalledPackageIndexDenial> {
        let record = self.packages.get(owner).ok_or_else(|| {
            WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::DomainNotInstalled,
                owner,
            )
        })?;
        self.indexed_operation_lookups
            .fetch_add(1, Ordering::Relaxed);
        let semantics = self
            .definitions
            .get(&(
                WorthQueryPortableDefinitionKind::GraphReadOperation,
                owner.to_string(),
                operation_slot.to_string(),
            ))
            .map(WorthQueryPortableDefinition::semantics)
            .ok_or_else(|| {
                WorthQueryInstalledPackageIndexDenial::new(
                    WorthQueryInstalledPackageIndexDenialKind::OperationNotInstalled,
                    operation_slot,
                )
            })?;
        Ok(WorthQueryInstalledOperationAuthority {
            runtime_ordinal: self.runtime.ordinal(),
            generation: self.generation,
            owner: owner.to_string(),
            package_identity: record.package.package().identity().clone(),
            admission_identity: record.package.admission_identity().clone(),
            package_authority_key: record.authority_key.clone(),
            operation_slot: operation_slot.to_string(),
            operation_semantics: semantics.to_string(),
        })
    }

    pub fn domain_operation(
        &self,
        owner: &str,
        operation_slot: &str,
    ) -> Result<WorthQueryInstalledDomainOperationAuthority, WorthQueryInstalledPackageIndexDenial>
    {
        let record = self.packages.get(owner).ok_or_else(|| {
            WorthQueryInstalledPackageIndexDenial::new(
                WorthQueryInstalledPackageIndexDenialKind::DomainNotInstalled,
                owner,
            )
        })?;
        self.indexed_operation_lookups
            .fetch_add(1, Ordering::Relaxed);
        let validated = self
            .domain_operations
            .get(&(owner.to_string(), operation_slot.to_string()))
            .cloned()
            .ok_or_else(|| {
                WorthQueryInstalledPackageIndexDenial::new(
                    WorthQueryInstalledPackageIndexDenialKind::OperationNotInstalled,
                    operation_slot,
                )
            })?;
        Ok(WorthQueryInstalledDomainOperationAuthority {
            runtime_ordinal: self.runtime.ordinal(),
            generation: self.generation,
            owner: owner.to_string(),
            package_identity: record.package.package().identity().clone(),
            admission_identity: record.package.admission_identity().clone(),
            package_authority_key: record.authority_key.clone(),
            validated,
        })
    }

    pub fn rebuild(&self) -> Self {
        Self::build_with_authority_root(
            self.runtime.retained(),
            self.generation,
            self.packages.values().map(|record| record.package.clone()),
            self.authority_root.clone(),
        )
        .expect("an admitted installed package set must rebuild without conflict")
    }

    pub fn successor_generation(&self) -> Self {
        Self::build_with_authority_root(
            self.runtime.retained(),
            self.generation.successor(),
            self.packages.values().map(|record| record.package.clone()),
            self.authority_root.clone(),
        )
        .expect("an admitted installed package set must advance without conflict")
    }

    pub fn rebuild_report(&self) -> WorthQueryInstalledPackageIndexRebuildReport {
        let rebuilt = self.rebuild();
        WorthQueryInstalledPackageIndexRebuildReport::new(
            self.identity.clone(),
            rebuilt.identity.clone(),
            rebuilt.counters,
        )
    }
}

#[cfg(test)]
mod relation_tests;
#[cfg(test)]
mod tests;
