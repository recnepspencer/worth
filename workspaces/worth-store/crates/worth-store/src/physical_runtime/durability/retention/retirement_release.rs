/// Identity of the exact free-map publication authorized by an extent release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetirementReleaseProjection {
    source_generation: u64,
    candidate_generation: u64,
    candidate_digest: [u8; 32],
    metadata_bytes: u64,
    publication: u64,
}

impl RetirementReleaseProjection {
    pub(in crate::physical_runtime) fn new(
        source_generation: u64,
        candidate_generation: u64,
        candidate_digest: [u8; 32],
        metadata_bytes: u64,
        publication: u64,
    ) -> Option<Self> {
        (source_generation != 0
            && source_generation.checked_add(1) == Some(candidate_generation)
            && candidate_digest != [0; 32]
            && metadata_bytes != 0
            && publication != 0)
            .then_some(Self {
                source_generation,
                candidate_generation,
                candidate_digest,
                metadata_bytes,
                publication,
            })
    }
    pub const fn source_generation(self) -> u64 {
        self.source_generation
    }
    pub const fn candidate_generation(self) -> u64 {
        self.candidate_generation
    }
    pub const fn candidate_digest(self) -> [u8; 32] {
        self.candidate_digest
    }
    pub const fn metadata_bytes(self) -> u64 {
        self.metadata_bytes
    }
    pub const fn publication(self) -> u64 {
        self.publication
    }
}
