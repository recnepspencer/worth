use worth_store_physical_format::{BlobSessionDeclarationV1, PersistedRecordIdentity};

use super::BlobResumeFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::blob) enum SelectedResumeClaim {
    Chunk {
        ordinal: u64,
        record: PersistedRecordIdentity,
        digest: [u8; 32],
        bytes: u64,
    },
    ReusedChunk {
        ordinal: u64,
        record: PersistedRecordIdentity,
        digest: [u8; 32],
        bytes: u64,
    },
    Node {
        ordinal: u64,
        record: PersistedRecordIdentity,
        frame_digest: [u8; 32],
    },
    Frontier {
        ordinal: u64,
        durable_bytes: u64,
        last_record: PersistedRecordIdentity,
        last_digest: [u8; 32],
    },
}

impl SelectedResumeClaim {
    pub(in crate::physical_runtime::blob) fn key(self) -> (u8, u64) {
        match self {
            Self::Chunk { ordinal, .. } | Self::ReusedChunk { ordinal, .. } => (0, ordinal),
            Self::Node { ordinal, .. } => (1, ordinal),
            Self::Frontier { ordinal, .. } => (2, ordinal),
        }
    }

    pub(in crate::physical_runtime::blob) fn chunk(
        self,
    ) -> Option<(u64, PersistedRecordIdentity, [u8; 32], u64)> {
        match self {
            Self::Chunk {
                ordinal,
                record,
                digest,
                bytes,
            }
            | Self::ReusedChunk {
                ordinal,
                record,
                digest,
                bytes,
            } => Some((ordinal, record, digest, bytes)),
            _ => None,
        }
    }
}

/// Selected C5 order is by RecordId, not chunk occurrence ordinal. Sorting
/// this one admitted vector makes gaps, duplicate ordinals and regressing
/// frontier bindings decidable before any resumed append is allowed.
pub(in crate::physical_runtime::blob) fn validate_selected_claims(
    claims: &mut [SelectedResumeClaim],
    declaration: BlobSessionDeclarationV1,
) -> Result<(), BlobResumeFailure> {
    claims.sort_unstable_by_key(|claim| claim.key());
    for pair in claims.windows(2) {
        if pair[0].key() == pair[1].key() {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
    }

    let chunk_size = u64::from(declaration.chunk_size());
    let total = declaration.declared_bytes();
    let maximum_chunks = total.div_ceil(chunk_size);
    let mut selected_chunks = 0_u64;
    for claim in claims.iter().copied() {
        let Some((ordinal, _, _, bytes)) = claim.chunk() else {
            continue;
        };
        if ordinal != selected_chunks || ordinal >= maximum_chunks {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
        let start = ordinal
            .checked_mul(chunk_size)
            .ok_or(BlobResumeFailure::ConflictingClaims)?;
        let expected = (total - start).min(chunk_size);
        if bytes != expected {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
        selected_chunks += 1;
    }

    for claim in claims.iter().copied() {
        let SelectedResumeClaim::Frontier {
            ordinal,
            durable_bytes,
            last_record,
            last_digest,
        } = claim
        else {
            continue;
        };
        if ordinal == 0 || ordinal > selected_chunks {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
        let expected_bytes = ordinal
            .checked_mul(chunk_size)
            .unwrap_or(u64::MAX)
            .min(total);
        if durable_bytes != expected_bytes {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
        let index =
            usize::try_from(ordinal - 1).map_err(|_| BlobResumeFailure::ConflictingClaims)?;
        let Some((_, record, digest, _)) = claims
            .get(index)
            .copied()
            .and_then(SelectedResumeClaim::chunk)
        else {
            return Err(BlobResumeFailure::ConflictingClaims);
        };
        if last_record != record || last_digest != digest {
            return Err(BlobResumeFailure::ConflictingClaims);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
