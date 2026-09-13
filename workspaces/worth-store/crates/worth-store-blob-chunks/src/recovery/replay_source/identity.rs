use worth_store_physical_backend::BackendTargetProfile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobReplaySourceIdentityKind {
    WalFrame,
    Checkpoint,
    Manifest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobReplaySourceIdentityDenial {
    EmptyDigest,
    ReversedLsnRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlobReplaySourceIdentity {
    kind: BlobReplaySourceIdentityKind,
    profile: BackendTargetProfile,
    digest: String,
    first_lsn: u64,
    last_lsn: u64,
}

impl BlobReplaySourceIdentity {
    pub fn new(
        kind: BlobReplaySourceIdentityKind,
        profile: BackendTargetProfile,
        digest: impl Into<String>,
        first_lsn: u64,
        last_lsn: u64,
    ) -> Result<Self, BlobReplaySourceIdentityDenial> {
        let digest = digest.into();
        if digest.is_empty() {
            return Err(BlobReplaySourceIdentityDenial::EmptyDigest);
        }
        if first_lsn > last_lsn {
            return Err(BlobReplaySourceIdentityDenial::ReversedLsnRange);
        }
        Ok(Self {
            kind,
            profile,
            digest,
            first_lsn,
            last_lsn,
        })
    }

    pub const fn kind(&self) -> BlobReplaySourceIdentityKind {
        self.kind
    }

    pub const fn profile(&self) -> BackendTargetProfile {
        self.profile
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub const fn first_lsn(&self) -> u64 {
        self.first_lsn
    }

    pub const fn last_lsn(&self) -> u64 {
        self.last_lsn
    }
}
