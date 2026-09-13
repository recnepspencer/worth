#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReplaySourceOutcomeKind {
    WalCheckpointManifestAdmitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReplaySourceOutcome {
    kind: BlobReplaySourceOutcomeKind,
}

impl BlobReplaySourceOutcome {
    pub const fn admitted() -> Self {
        Self {
            kind: BlobReplaySourceOutcomeKind::WalCheckpointManifestAdmitted,
        }
    }

    pub const fn kind(self) -> BlobReplaySourceOutcomeKind {
        self.kind
    }
}
