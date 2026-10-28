//! Fixed-width artifact header: layout, emission and verification. The
//! verification order is cost-ordered and version-first so bytes from any
//! other format version refuse as unknown rather than as malformed.

use super::payload_digest::RelationalCanonicalCommitArtifactDigest as Digest;
use super::RelationalCanonicalCommitArtifactFrameRefusal as Refusal;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;

pub(super) const MAGIC: [u8; 8] = *b"WRTHRCCA";
pub(super) const FORMAT_VERSION: u16 = 1;

const VERSION_OFFSET: usize = MAGIC.len();
const POSITION_OFFSET: usize = VERSION_OFFSET + 2;
const COMMIT_OFFSET: usize = POSITION_OFFSET + 8;
pub(super) const DIGEST_OFFSET: usize = COMMIT_OFFSET + 8;
const LENGTH_OFFSET: usize = DIGEST_OFFSET + Digest::LEN;
pub(super) const HEADER_LEN: usize = LENGTH_OFFSET + 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CanonicalCommitArtifactHeader {
    pub(super) position: PatchStreamPosition,
    pub(super) commit_id: CommitId,
    pub(super) digest: Digest,
}

impl CanonicalCommitArtifactHeader {
    /// Header bytes for a payload of `payload_len` bytes. Encoding writes the
    /// digest slot last, once the payload exists, so callers pass a
    /// placeholder digest here and overwrite `DIGEST_OFFSET` afterwards.
    pub(super) fn emit(&self, payload_len: u64) -> [u8; HEADER_LEN] {
        let mut header = [0_u8; HEADER_LEN];
        header[..VERSION_OFFSET].copy_from_slice(&MAGIC);
        header[VERSION_OFFSET..POSITION_OFFSET].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        header[POSITION_OFFSET..COMMIT_OFFSET].copy_from_slice(&self.position.0.to_le_bytes());
        header[COMMIT_OFFSET..DIGEST_OFFSET].copy_from_slice(&self.commit_id.0.to_le_bytes());
        header[DIGEST_OFFSET..LENGTH_OFFSET].copy_from_slice(self.digest.as_bytes());
        header[LENGTH_OFFSET..].copy_from_slice(&payload_len.to_le_bytes());
        header
    }

    /// Verify one complete frame and return its header with the payload it
    /// covers. Nothing after the declared payload is accepted.
    ///
    /// Cost: O(1) header checks, then one SHA-256 pass over the payload.
    pub(super) fn verify(frame: &[u8]) -> Result<(Self, &[u8]), Refusal> {
        let magic = frame
            .get(..VERSION_OFFSET)
            .ok_or(Refusal::TruncatedHeader {
                available: frame.len(),
            })?;
        if magic != MAGIC {
            return Err(Refusal::UnrecognizedMagic);
        }
        let version = u16::from_le_bytes(fixed(frame, VERSION_OFFSET)?);
        if version != FORMAT_VERSION {
            return Err(Refusal::UnsupportedVersion {
                found: version,
                supported: FORMAT_VERSION,
            });
        }
        let header = Self {
            position: PatchStreamPosition(u64::from_le_bytes(fixed(frame, POSITION_OFFSET)?)),
            commit_id: CommitId(u64::from_le_bytes(fixed(frame, COMMIT_OFFSET)?)),
            digest: Digest::recorded(fixed(frame, DIGEST_OFFSET)?),
        };
        let declared = u64::from_le_bytes(fixed(frame, LENGTH_OFFSET)?);
        let payload = declared_payload(frame, declared)?;
        let computed = Digest::of_payload(payload);
        if computed != header.digest {
            return Err(Refusal::DigestMismatch {
                recorded: header.digest,
                computed,
            });
        }
        Ok((header, payload))
    }
}

fn fixed<const N: usize>(frame: &[u8], offset: usize) -> Result<[u8; N], Refusal> {
    frame
        .get(offset..offset + N)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(Refusal::TruncatedHeader {
            available: frame.len(),
        })
}

fn declared_payload(frame: &[u8], declared: u64) -> Result<&[u8], Refusal> {
    let payload = &frame[HEADER_LEN..];
    let available = payload.len() as u64;
    if available < declared {
        return Err(Refusal::TruncatedPayload {
            declared,
            available,
        });
    }
    if available > declared {
        return Err(Refusal::TrailingBytes {
            declared,
            available,
        });
    }
    Ok(payload)
}
