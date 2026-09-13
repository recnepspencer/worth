use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogGeneration, PhysicalRecordFormatDeclaration,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedBootstrapCatalog<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    current_root_generation: CurrentRootCatalogGeneration,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedBootstrapCatalog<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        catalog: BootstrapCatalog,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_bootstrap_catalog()
            || catalog.store_identity() != scope.store_identity()
            || catalog.format() != scope.record_format()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.bootstrap_exact_scope_digest()),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            record_format: catalog.format(),
            current_root_generation: catalog.current_root().generation(),
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub const fn current_root_generation(&self) -> CurrentRootCatalogGeneration {
        self.current_root_generation
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
