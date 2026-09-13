use worth_store_physical_format::{
    FreeSpaceBlockReference, FreeSpaceMembershipBlockScopeIdentity,
    PhysicalFreeSpaceMembershipBlock, PhysicalRecordFormatDeclaration,
    RecordFreeSpaceManifestEntry,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedFreeSpaceMembershipBlock<'media> {
    scope: PhysicalArtifactScope,
    identity: FreeSpaceMembershipBlockScopeIdentity,
    record_format: PhysicalRecordFormatDeclaration,
    block: PhysicalFreeSpaceMembershipBlock,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedFreeSpaceMembershipBlock<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        block: PhysicalFreeSpaceMembershipBlock,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let identity = scope.free_space_membership_block_identity()?;
        if block.tree_identity() != identity.tree().get()
            || block.reference(validated_range_checksum) != identity.reference()
            || record_format != scope.record_format()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(scope.free_space_exact_scope_digest()?),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            identity,
            record_format,
            block,
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn identity(&self) -> FreeSpaceMembershipBlockScopeIdentity {
        self.identity
    }

    pub const fn reference(&self) -> FreeSpaceBlockReference {
        self.identity.reference()
    }

    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
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

    pub fn entries(&self) -> Option<&[RecordFreeSpaceManifestEntry]> {
        self.block.entries()
    }

    pub fn children(&self) -> Option<&[FreeSpaceBlockReference]> {
        self.block.children()
    }

    pub fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
