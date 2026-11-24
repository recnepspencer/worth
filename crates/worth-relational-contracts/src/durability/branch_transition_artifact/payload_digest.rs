use sha2::{Digest as _, Sha256};

/// SHA-256 of one artifact payload. It identifies the transition bytes a medium
/// holds at a sequence; it is not a digest of the header.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelationalBranchTransitionArtifactDigest([u8; Self::LEN]);

impl RelationalBranchTransitionArtifactDigest {
    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }
}

impl std::fmt::Debug for RelationalBranchTransitionArtifactDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("sha256:")?;
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}

/// Operations used by the Relational owners.
pub trait RelationalBranchTransitionArtifactDigestOwnerPort: Sized {
    const LEN: usize;
    /// Cost: one SHA-256 pass over `payload`.
    fn of_payload(payload: &[u8]) -> Self;
    /// The value a frame header records; it is trusted only after it equals
    /// [`Self::of_payload`] for the payload it covers.
    fn recorded(
        bytes: [u8; <RelationalBranchTransitionArtifactDigest as RelationalBranchTransitionArtifactDigestOwnerPort>::LEN],
    ) -> Self;
    /// Slot value written before the payload exists; encoding overwrites it.
    fn unsealed() -> Self;
}

impl RelationalBranchTransitionArtifactDigestOwnerPort
    for RelationalBranchTransitionArtifactDigest
{
    const LEN: usize = 32;

    fn of_payload(payload: &[u8]) -> Self {
        Self(Sha256::digest(payload).into())
    }

    fn recorded(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    fn unsealed() -> Self {
        Self([0; Self::LEN])
    }
}
