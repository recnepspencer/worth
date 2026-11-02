use sha2::{Digest as _, Sha256};

/// SHA-256 of one artifact payload. It identifies the commit bytes a medium
/// holds at a position; it is not a digest of the header.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelationalCanonicalCommitArtifactDigest([u8; Self::LEN]);

impl RelationalCanonicalCommitArtifactDigest {
    pub(super) const LEN: usize = 32;

    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    /// Cost: one SHA-256 pass over `payload`.
    pub(super) fn of_payload(
        position: u64,
        commit: u64,
        payload: &[u8],
    ) -> Result<Self, super::RelationalCanonicalCommitArtifactFrameRefusal> {
        use super::borrowed_payload::{malformed, BorrowedPayload};
        let mut leaves = [[0; 32]; 3];
        let mut seen = 0u8;
        BorrowedPayload(payload)
            .frame_fields(|name, value| {
                let index = match name {
                    "position" => 0,
                    "canonical" => 1,
                    "original_inputs" => 2,
                    _ => return Err(malformed()),
                };
                let bit = 1 << index;
                if seen & bit != 0 {
                    return Err(malformed());
                }
                seen |= bit;
                leaves[index] = Sha256::digest(value.0).into();
                Ok(())
            })
            .map_err(|_| {
                super::RelationalCanonicalCommitArtifactFrameRefusal::InvalidPayloadShape
            })?;
        if seen != 7 {
            return Err(super::RelationalCanonicalCommitArtifactFrameRefusal::InvalidPayloadShape);
        }
        Ok(Self::from_leaves(
            position, commit, leaves[0], leaves[1], leaves[2],
        ))
    }

    pub(crate) fn from_leaves(
        position: u64,
        commit: u64,
        position_leaf: [u8; 32],
        envelope_leaf: [u8; 32],
        inputs_leaf: [u8; 32],
    ) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth.relational.canonical.v5\0");
        digest.update(position.to_le_bytes());
        digest.update(commit.to_le_bytes());
        digest.update(position_leaf);
        digest.update(envelope_leaf);
        digest.update(inputs_leaf);
        Self(digest.finalize().into())
    }

    /// The value a frame header records; it is trusted only after it equals
    /// [`Self::of_payload`] for the payload it covers.
    pub(super) fn recorded(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }

    /// Slot value written before the payload exists; encoding overwrites it.
    pub(super) const fn unsealed() -> Self {
        Self([0; Self::LEN])
    }
}

impl std::fmt::Debug for RelationalCanonicalCommitArtifactDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("sha256:")?;
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}
