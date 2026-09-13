use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableFreeSpaceManifestHeader, FreeSpaceBlockReference,
    FreeSpaceHeaderScopeIdentity, PhysicalRecordFormatDeclaration,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedFreeSpaceHeader<'media> {
    scope: PhysicalArtifactScope,
    identity: FreeSpaceHeaderScopeIdentity,
    record_format: PhysicalRecordFormatDeclaration,
    node_capacity: u16,
    segment_page_capacity: u32,
    entry_count: u64,
    next_segment: u64,
    next_page: u64,
    next_extent: u64,
    next_block: u64,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedFreeSpaceHeader<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        header: DurableFreeSpaceManifestHeader,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        let identity = scope.free_space_header_identity()?;
        if header.generation() != identity.generation().get()
            || header.tree_identity() != identity.tree().get()
            || header.root() != identity.root()
            || validated_range_checksum != identity.complete_child_checksum().get()
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
            node_capacity: header.node_capacity(),
            segment_page_capacity: header.segment_page_capacity(),
            entry_count: header.entry_count(),
            next_segment: header.next_segment(),
            next_page: header.next_page(),
            next_extent: header.next_extent(),
            next_block: header.next_block(),
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn identity(&self) -> FreeSpaceHeaderScopeIdentity {
        self.identity
    }

    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub const fn root(&self) -> Option<FreeSpaceBlockReference> {
        self.identity.root()
    }

    pub const fn complete_child_checksum(&self) -> DurableArtifactCrc32c {
        self.identity.complete_child_checksum()
    }

    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }

    pub const fn segment_page_capacity(&self) -> u32 {
        self.segment_page_capacity
    }

    pub const fn entry_count(&self) -> u64 {
        self.entry_count
    }

    pub const fn next_segment(&self) -> u64 {
        self.next_segment
    }

    pub const fn next_page(&self) -> u64 {
        self.next_page
    }

    pub const fn next_extent(&self) -> u64 {
        self.next_extent
    }

    pub const fn next_block(&self) -> u64 {
        self.next_block
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
