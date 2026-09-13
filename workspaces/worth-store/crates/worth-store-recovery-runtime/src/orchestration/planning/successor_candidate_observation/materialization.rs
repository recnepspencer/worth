use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};

#[derive(Default)]
pub(super) struct CandidateMaterialization {
    retained_bytes: u64,
    retained_artifacts: u64,
    retained_references: u64,
    root_structures: u64,
    free_space_headers: u64,
    placements: u64,
    segment_entries: u64,
    free_entries: u64,
    peak_bytes: u64,
}

impl CandidateMaterialization {
    pub(super) fn retain_root(&mut self, bytes: usize) {
        self.root_structures = self.root_structures.saturating_add(1);
        self.retain_artifact(bytes);
    }

    pub(super) fn retain_free_space_header(&mut self, bytes: usize) {
        self.free_space_headers = self.free_space_headers.saturating_add(1);
        self.retain_artifact(bytes);
    }

    pub(super) fn retain_artifact(&mut self, bytes: usize) {
        self.retained_bytes = self.retained_bytes.saturating_add(bytes as u64);
        self.retained_artifacts = self.retained_artifacts.saturating_add(1);
        self.capture_peak();
    }

    pub(super) fn retain_reference(&mut self) {
        self.retained_references = self.retained_references.saturating_add(1);
        self.capture_peak();
    }

    pub(super) fn retain_placements(&mut self, entries: usize) {
        self.placements = self.placements.saturating_add(entries as u64);
        self.capture_peak();
    }

    pub(super) fn retain_segment_entries(&mut self, entries: usize) {
        self.segment_entries = self.segment_entries.saturating_add(entries as u64);
        self.capture_peak();
    }

    pub(super) fn retain_free_entries(&mut self, entries: usize) {
        self.free_entries = self.free_entries.saturating_add(entries as u64);
        self.capture_peak();
    }

    pub(super) const fn peak_bytes(&self) -> u64 {
        self.peak_bytes
    }

    fn capture_peak(&mut self) {
        let descriptor_bytes = std::mem::size_of::<RecordArtifactFile>()
            .saturating_add(std::mem::size_of::<Box<[u8]>>()) as u64;
        let current = self
            .retained_bytes
            .saturating_add(
                self.root_structures
                    .saturating_mul(std::mem::size_of::<DurablePhysicalRootManifest>() as u64),
            )
            .saturating_add(
                self.free_space_headers
                    .saturating_mul(std::mem::size_of::<DurableFreeSpaceManifestHeader>() as u64),
            )
            .saturating_add(
                self.placements
                    .saturating_mul(std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64),
            )
            .saturating_add(
                self.segment_entries
                    .saturating_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64),
            )
            .saturating_add(
                self.free_entries
                    .saturating_mul(std::mem::size_of::<RecordFreeSpaceManifestEntry>() as u64),
            )
            .saturating_add(self.retained_artifacts.saturating_mul(descriptor_bytes));
        let current = current.saturating_add(
            self.retained_references
                .saturating_mul(std::mem::size_of::<RecordArtifactFile>() as u64),
        );
        self.peak_bytes = self.peak_bytes.max(current);
    }
}
