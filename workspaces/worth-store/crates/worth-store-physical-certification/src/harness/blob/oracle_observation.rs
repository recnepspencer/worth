#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobHarnessOracleObservation {
    byte_equality_verified: bool,
    chunk_ordering_verified: bool,
    digest_checksum_distinction_verified: bool,
    reachability_verified: bool,
    cross_scope_dedupe_guarded: bool,
    constant_memory_envelope_held: bool,
    no_sidecar_path_verified: bool,
    heavy_evidence_verified: bool,
    heavy_cleanup_verified: bool,
    heavy_pattern_lane_verified: bool,
}

impl BlobHarnessOracleObservation {
    pub const fn byte_equality_verified(self) -> bool {
        self.byte_equality_verified
    }

    pub const fn chunk_ordering_verified(self) -> bool {
        self.chunk_ordering_verified
    }

    pub const fn digest_checksum_distinction_verified(self) -> bool {
        self.digest_checksum_distinction_verified
    }

    pub const fn reachability_verified(self) -> bool {
        self.reachability_verified
    }

    pub const fn cross_scope_dedupe_guarded(self) -> bool {
        self.cross_scope_dedupe_guarded
    }

    pub const fn constant_memory_envelope_held(self) -> bool {
        self.constant_memory_envelope_held
    }

    pub const fn no_sidecar_path_verified(self) -> bool {
        self.no_sidecar_path_verified
    }

    pub const fn heavy_evidence_verified(self) -> bool {
        self.heavy_evidence_verified
    }

    pub const fn heavy_cleanup_verified(self) -> bool {
        self.heavy_cleanup_verified
    }

    pub const fn heavy_pattern_lane_verified(self) -> bool {
        self.heavy_pattern_lane_verified
    }
}
