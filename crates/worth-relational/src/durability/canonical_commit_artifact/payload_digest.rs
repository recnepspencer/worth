use sha2::{Digest as _, Sha256};

const DIGEST_DOMAIN: &[u8] = b"worth.relational.canonical.v8\0";

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
        let leaves = payload_leaves(payload)?;
        Ok(Self::from_leaves(
            position,
            commit,
            Sha256::digest(leaves[0].0).into(),
            Sha256::digest(leaves[1].0).into(),
            Sha256::digest(leaves[2].0).into(),
        ))
    }

    pub(super) fn of_payload_for_request(
        position: u64,
        commit: u64,
        payload: &[u8],
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Self, RequestDigestDenial> {
        let fields = payload_leaves(payload).map_err(RequestDigestDenial::Frame)?;
        let mut leaves = [[0; 32]; 3];
        for (field, leaf) in fields.iter().zip(leaves.iter_mut()) {
            let mut hash = Sha256::new();
            for chunk in field.0.chunks(4096) {
                crate::execution::request_stage::checkpoint(context, chunk.len() as u64)
                    .map_err(|cause| RequestDigestDenial::Request(cause.into()))?;
                hash.update(chunk);
            }
            *leaf = hash.finalize().into();
        }
        Self::from_leaves_for_request(position, commit, leaves[0], leaves[1], leaves[2], context)
            .map_err(RequestDigestDenial::Request)
    }

    pub(super) fn from_leaves_for_request(
        position: u64,
        commit: u64,
        position_leaf: [u8; 32],
        envelope_leaf: [u8; 32],
        inputs_leaf: [u8; 32],
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        let bytes = DIGEST_DOMAIN.len() + 2 * std::mem::size_of::<u64>() + 3 * Self::LEN;
        crate::execution::request_stage::checkpoint(context, bytes as u64)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
        Ok(Self::from_leaves(
            position,
            commit,
            position_leaf,
            envelope_leaf,
            inputs_leaf,
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
        digest.update(DIGEST_DOMAIN);
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

pub(super) enum RequestDigestDenial {
    Frame(super::RelationalCanonicalCommitArtifactFrameRefusal),
    Request(crate::execution::RelationalExecutionDenialCause),
}

fn payload_leaves(
    payload: &[u8],
) -> Result<
    [super::borrowed_payload::BorrowedPayload<'_>; 3],
    super::RelationalCanonicalCommitArtifactFrameRefusal,
> {
    use super::borrowed_payload::{malformed, BorrowedPayload};
    use super::RelationalCanonicalCommitArtifactFrameRefusal as Refusal;
    let mut leaves = [BorrowedPayload(&[]); 3];
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
            leaves[index] = value;
            Ok(())
        })
        .map_err(|_| Refusal::InvalidPayloadShape)?;
    if seen != 7 {
        return Err(Refusal::InvalidPayloadShape);
    }
    Ok(leaves)
}
