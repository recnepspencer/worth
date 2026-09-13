use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PhysicalRecordFormatDeclaration,
    PhysicalRootRoutingBlock,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedRootRoutingBlock<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    block: PhysicalRootRoutingBlock,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedRootRoutingBlock<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        block: PhysicalRootRoutingBlock,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let expected = scope.root_routing_block_identity()?;
        if record_format != scope.record_format()
            || block.tree_identity() != expected.tree().get()
            || block.reference(validated_range_checksum) != expected.reference()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.root_routing_exact_scope_digest()),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            record_format,
            block,
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

    pub const fn tree_identity(&self) -> u64 {
        self.block.tree_identity()
    }

    pub const fn generation(&self) -> u64 {
        self.block.generation()
    }

    pub const fn block_identity(&self) -> u64 {
        self.block.block()
    }

    pub const fn level(&self) -> u16 {
        self.block.level()
    }

    pub fn entries(&self) -> Option<&[CurrentPhysicalRecordPlacement]> {
        self.block.entries()
    }

    pub fn children(&self) -> Option<&[ManifestBlockReference]> {
        self.block.children()
    }

    pub fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
