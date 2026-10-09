use super::*;

impl DurablePhysicalRootManifestBuilder {
    pub const fn record_count(mut self, record_count: u64) -> Self {
        self.record_count = record_count;
        self
    }
    pub const fn next_block(mut self, next_block: u64) -> Self {
        self.next_block = next_block;
        self
    }
    pub const fn next_segment_block(mut self, next_segment_block: u64) -> Self {
        self.next_segment_block = next_segment_block;
        self
    }
    pub const fn routing_root(mut self, routing_root: Option<ManifestBlockReference>) -> Self {
        self.routing_root = routing_root;
        self
    }
    pub const fn segment_root(
        mut self,
        segment_root: Option<SegmentManifestBlockReference>,
    ) -> Self {
        self.segment_root = segment_root;
        self
    }
    pub const fn free_space_root(
        mut self,
        free_space_root: Option<FreeSpaceBlockReference>,
    ) -> Self {
        self.free_space_root = free_space_root;
        self
    }
    pub const fn release_custody_head_root(
        mut self,
        root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    ) -> Self {
        self.release_custody_head_root = root;
        self
    }
    pub const fn next_release_custody_head_block(mut self, next: u64) -> Self {
        self.next_release_custody_head_block = next;
        self
    }
    pub const fn derived_family_directory(
        mut self,
        directory: Option<DerivedFamilyRootDirectoryBinding>,
    ) -> Self {
        self.derived_family_directory = directory;
        self
    }
    pub const fn latest_blob_publication(
        mut self,
        publication: Option<IndexedThroughBlobPublication>,
    ) -> Self {
        self.latest_blob_publication = publication;
        self
    }
    pub const fn latest_blob_quarantine(
        mut self,
        quarantine: Option<PersistedRecordIdentity>,
    ) -> Self {
        self.latest_blob_quarantine = quarantine;
        self
    }
    pub const fn tier_epoch_anchor(mut self, anchor: Option<[u8; 32]>) -> Self {
        self.tier_epoch_anchor = anchor;
        self
    }
    pub const fn last_inline_record(
        mut self,
        last_inline_record: Option<PersistedRecordIdentity>,
    ) -> Self {
        self.last_inline_record = last_inline_record;
        self
    }
    pub const fn last_inline_segment(
        mut self,
        last_inline_segment: Option<SegmentGenerationCell>,
    ) -> Self {
        self.last_inline_segment = last_inline_segment;
        self
    }

    pub fn admit(self) -> Option<DurablePhysicalRootManifest> {
        let Self {
            generation,
            tree_identity,
            node_capacity,
            free_space_checksum,
            record_count,
            next_block,
            next_segment_block,
            routing_root,
            segment_root,
            free_space_root,
            release_custody_head_root,
            next_release_custody_head_block,
            latest_blob_publication,
            latest_blob_quarantine,
            tier_epoch_anchor,
            derived_family_directory,
            last_inline_record,
            last_inline_segment,
        } = self;
        let generation = PhysicalGeneration::from_raw(generation).ok()?;
        let root_reference = PhysicalRootReference::from_raw(generation.get()).ok()?;
        let tail_shape_is_valid = last_inline_record.is_some() == last_inline_segment.is_some();
        let shape_is_valid = match (record_count, routing_root) {
            (0, None) => last_inline_record.is_none() && last_inline_segment.is_none(),
            (0, Some(_)) | (_, None) => false,
            (_, Some(reference)) => {
                required_tree_level(record_count, node_capacity) == Some(reference.level())
                    && reference.generation() <= generation.get()
                    && reference.block() < next_block
                    && last_inline_record.is_none_or(|record| reference.contains(record))
            }
        };
        let segment_shape_is_valid = segment_root.is_none_or(|reference| {
            required_tree_level(record_count, node_capacity)
                .is_some_and(|maximum| reference.level() <= maximum)
                && reference.generation() <= generation.get()
                && reference.block() < next_segment_block
        });
        let free_space_shape_is_valid =
            free_space_root.is_none_or(|reference| reference.generation() <= generation.get());
        let head_shape_is_valid = release_custody_head_root.is_none_or(|reference| {
            reference.generation() <= generation.get()
                && reference.block() < next_release_custody_head_block
        });
        let directory_shape_is_valid = derived_family_directory.is_none_or(|binding| {
            routing_root.is_some_and(|reference| reference.contains(binding.directory_record()))
                && binding
                    .indexed_through_blob_publication()
                    .is_none_or(|publication| {
                        publication.root_generation() <= generation.get()
                            && routing_root
                                .is_some_and(|reference| reference.contains(publication.record()))
                    })
        });
        let publication_shape_is_valid = latest_blob_publication.is_none_or(|publication| {
            publication.root_generation() <= generation.get()
                && routing_root.is_some_and(|reference| reference.contains(publication.record()))
        });
        let quarantine_shape_is_valid = latest_blob_quarantine
            .is_none_or(|record| routing_root.is_some_and(|reference| reference.contains(record)));
        if tree_identity == 0
            || node_capacity < 2
            || next_block == 0
            || next_segment_block == 0
            || next_release_custody_head_block == 0
            || free_space_checksum == 0
            || !shape_is_valid
            || !tail_shape_is_valid
            || !segment_shape_is_valid
            || !free_space_shape_is_valid
            || !head_shape_is_valid
            || !directory_shape_is_valid
            || !publication_shape_is_valid
            || !quarantine_shape_is_valid
            || tier_epoch_anchor == Some([0; 32])
        {
            return None;
        }
        Some(DurablePhysicalRootManifest {
            root: PhysicalGenerationAuthority::for_canonical_physical_format()
                .root_publication_cell(root_reference)
                .with_root_publication_generation(generation),
            tree_identity,
            node_capacity,
            record_count,
            next_block,
            next_segment_block,
            free_space_checksum,
            routing_root,
            segment_root,
            free_space_root,
            release_custody_head_root,
            next_release_custody_head_block,
            latest_blob_publication,
            latest_blob_quarantine,
            tier_epoch_anchor,
            derived_family_directory,
            last_inline_record,
            last_inline_segment,
            requires_maintenance_protocol: tier_epoch_anchor.is_some(),
        })
    }
}
