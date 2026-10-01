use super::*;

impl DurablePhysicalRootManifest {
    pub const fn generation(&self) -> u64 {
        self.root.generation().get()
    }
    pub const fn root_cell(&self) -> RootPublicationCell {
        self.root
    }
    pub const fn node_capacity(&self) -> u16 {
        self.node_capacity
    }
    pub const fn tree_identity(&self) -> u64 {
        self.tree_identity
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
    pub const fn last_inline_segment(&self) -> Option<SegmentGenerationCell> {
        self.last_inline_segment
    }
}
