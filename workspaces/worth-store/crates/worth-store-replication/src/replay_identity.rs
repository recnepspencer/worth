use worth_store_physical_backend::BackendTargetProfile;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityReplayKind {
    WalFrame,
    Checkpoint,
    Manifest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityReplayIdentityDenial {
    EmptyDigest,
    ReversedLsnRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurabilityReplayIdentity {
    kind: DurabilityReplayKind,
    profile: BackendTargetProfile,
    digest: String,
    first_lsn: u64,
    last_lsn: u64,
}

impl DurabilityReplayIdentity {
    pub fn new(
        kind: DurabilityReplayKind,
        profile: BackendTargetProfile,
        digest: impl Into<String>,
        first_lsn: u64,
        last_lsn: u64,
    ) -> Result<Self, DurabilityReplayIdentityDenial> {
        let digest = digest.into();
        if digest.is_empty() {
            return Err(DurabilityReplayIdentityDenial::EmptyDigest);
        }
        if first_lsn > last_lsn {
            return Err(DurabilityReplayIdentityDenial::ReversedLsnRange);
        }
        Ok(Self {
            kind,
            profile,
            digest,
            first_lsn,
            last_lsn,
        })
    }

    pub const fn kind(&self) -> DurabilityReplayKind {
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
