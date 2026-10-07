use worth_store_physical_format::{
    durable_root_manifest_frame_digest, DerivedFamilyRootDirectoryBinding,
    DurablePhysicalRootManifest, FreeSpaceBlockReference, IndexedThroughBlobPublication,
    ManifestBlockReference, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, SegmentGenerationCell, SegmentManifestBlockReference,
};

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedRootManifest<'media> {
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    tree_identity: u64,
    node_capacity: u16,
    record_count: u64,
    next_block: u64,
    next_segment_block: u64,
    free_space_checksum: u32,
    routing_root: Option<ManifestBlockReference>,
    segment_root: Option<SegmentManifestBlockReference>,
    free_space_root: Option<FreeSpaceBlockReference>,
    release_custody_head_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    next_release_custody_head_block: u64,
    latest_blob_publication: Option<IndexedThroughBlobPublication>,
    latest_blob_quarantine: Option<PersistedRecordIdentity>,
    tier_epoch_anchor: Option<[u8; 32]>,
    derived_family_directory: Option<DerivedFamilyRootDirectoryBinding>,
    last_inline_record: Option<PersistedRecordIdentity>,
    last_inline_segment: Option<SegmentGenerationCell>,
    requires_maintenance_protocol: bool,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedRootManifest<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        manifest: DurablePhysicalRootManifest,
        record_format: PhysicalRecordFormatDeclaration,
        validated_range_checksum: u32,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if !scope.is_root_manifest()
            || manifest.generation() != scope.root_generation()?
            || record_format != scope.record_format()
            || inspected.byte_count() != scope.byte_range().length()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::crc32c(
                scope.selector_or_manifest_exact_scope_digest(),
            ),
            PhysicalIntegrityValidationDigest::crc32c(validated_range_checksum),
            PhysicalIntegrityValidationMechanism::Crc32cV1,
        )?;
        Some(Self {
            scope,
            record_format,
            tree_identity: manifest.tree_identity(),
            node_capacity: manifest.node_capacity(),
            record_count: manifest.record_count(),
            next_block: manifest.next_block(),
            next_segment_block: manifest.next_segment_block(),
            free_space_checksum: manifest.free_space_checksum(),
            routing_root: manifest.routing_root(),
            segment_root: manifest.segment_root(),
            free_space_root: manifest.free_space_root(),
            release_custody_head_root: manifest.release_custody_head_root(),
            next_release_custody_head_block: manifest.next_release_custody_head_block(),
            latest_blob_publication: manifest.latest_blob_publication(),
            latest_blob_quarantine: manifest.latest_blob_quarantine(),
            tier_epoch_anchor: manifest.tier_epoch_anchor(),
            derived_family_directory: manifest.derived_family_directory(),
            last_inline_record: manifest.last_inline_record(),
            last_inline_segment: manifest.last_inline_segment(),
            requires_maintenance_protocol: manifest.requires_maintenance_protocol(),
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn root_generation(&self) -> u64 {
        match self.scope.root_generation() {
            Some(generation) => generation,
            None => unreachable!(),
        }
    }

    pub const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
    }

    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }

    pub const fn record_count(&self) -> u64 {
        self.record_count
    }

    pub const fn next_block(&self) -> u64 {
        self.next_block
    }

    pub const fn next_segment_block(&self) -> u64 {
        self.next_segment_block
    }

    pub const fn free_space_checksum(&self) -> u32 {
        self.free_space_checksum
    }

    pub const fn routing_root(&self) -> Option<ManifestBlockReference> {
        self.routing_root
    }

    pub const fn segment_root(&self) -> Option<SegmentManifestBlockReference> {
        self.segment_root
    }

    pub const fn free_space_root(&self) -> Option<FreeSpaceBlockReference> {
        self.free_space_root
    }

    pub const fn release_custody_head_root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.release_custody_head_root
    }

    pub const fn next_release_custody_head_block(&self) -> u64 {
        self.next_release_custody_head_block
    }

    pub const fn derived_family_directory(&self) -> Option<DerivedFamilyRootDirectoryBinding> {
        self.derived_family_directory
    }

    pub const fn latest_blob_publication(&self) -> Option<IndexedThroughBlobPublication> {
        self.latest_blob_publication
    }
    pub const fn latest_blob_quarantine(&self) -> Option<PersistedRecordIdentity> {
        self.latest_blob_quarantine
    }
    pub const fn tier_epoch_anchor(&self) -> Option<[u8; 32]> {
        self.tier_epoch_anchor
    }

    pub const fn last_inline_record(&self) -> Option<PersistedRecordIdentity> {
        self.last_inline_record
    }

    pub const fn requires_maintenance_protocol(&self) -> bool {
        self.requires_maintenance_protocol
    }

    pub const fn last_inline_segment(&self) -> Option<SegmentGenerationCell> {
        self.last_inline_segment
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }

    /// Exact complete frame identity of this independently admitted source root.
    pub fn frame_sha256(&self) -> [u8; 32] {
        durable_root_manifest_frame_digest(self.inspected.bytes())
    }
}
