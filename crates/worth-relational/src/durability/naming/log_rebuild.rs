//! Deterministic rebuild of the derived name index from the durable log, and
//! fresh-installation admission of retained names against it.
//!
//! The persisted frames are durable truth; the index is derived. Reopen,
//! crash recovery, and fresh installation all destroy nothing and trust
//! nothing: they decode, verify, and replay every frame through the index's
//! ordinary atomic batch ingestion.

use crate::history::data::BranchId;
use crate::lineage::authority::PersistedNameBytes;
use crate::lineage::data::{PersistentNameKey, PersistentNameSubjectKind};
use crate::lineage::index::PersistentNameBranchIndex;

use super::frame_decoding::{decode_commit_frame, decode_log_header, split_commit_frame};
use super::frame_readmission::{readmit_commit_frame, ReadmittedNames};
use super::refusal::PersistentNameDurabilityError as Refusal;

/// Rebuilds the derived name index from a complete durable log.
///
/// Refuses typed, returning no index at all, when the format version is
/// unsupported, a frame is torn or non-canonical, any persisted birth fails
/// digest re-verification, any identity is referenced without a verified
/// birth, or the index refuses a verified frame.
///
/// Complexity: O(log bytes + frames * per-batch ingestion).
pub fn rebuild_name_index_from_durable_log(
    bytes: &[u8],
) -> Result<PersistentNameBranchIndex, Refusal> {
    let (branch_id, frames) = decode_log_header(bytes)?;
    decode_and_rebuild_frames(branch_id, frames)
}

/// Decodes, verifies, and replays every frame after the log header.
fn decode_and_rebuild_frames(
    branch_id: BranchId,
    frames: &[u8],
) -> Result<PersistentNameBranchIndex, Refusal> {
    let mut index = PersistentNameBranchIndex::new(branch_id, None);
    let mut readmitted = ReadmittedNames::default();
    let mut rest = frames;
    while !rest.is_empty() {
        let (payload, after) = split_commit_frame(rest)?;
        let frame = decode_commit_frame(payload)?;
        let floor = frame.min_retained_commit;
        let batch = readmit_commit_frame(frame, &mut readmitted)?;
        index
            .apply(&batch)
            .map_err(|denial| Refusal::IndexRefused {
                commit: batch.commit(),
                denial,
            })?;
        if let Some(floor) = floor {
            index.advance_retention_floor(floor);
        }
        rest = after;
    }
    Ok(index)
}

/// Admits a retained 32-byte identity in a fresh installation only when the
/// rebuilt index recorded that name's verified persisted birth.
///
/// Never mints or forges: the admitted key is the one the index recorded
/// from a committed birth; bare bytes with no persisted birth refuse typed.
///
/// Complexity: O(1) expected.
pub fn admit_persisted_name(
    index: &PersistentNameBranchIndex,
    subject: PersistentNameSubjectKind,
    raw: PersistedNameBytes,
) -> Result<PersistentNameKey, Refusal> {
    index
        .recorded_name(subject, &raw)
        .ok_or(Refusal::UnknownPersistedName { subject, raw })
}
