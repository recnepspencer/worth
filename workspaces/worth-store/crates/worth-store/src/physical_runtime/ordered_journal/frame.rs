//! The journal record frame: a self-identifying header, the record's jump
//! pointers to earlier journal records, then the owner payload.
//!
//! The spine of the record at position `p` names, for every `k`, the record at
//! the largest multiple of `2^k` that is at most `p - 1`. Every such record is
//! either the predecessor head or a member of the head's own spine, so a new
//! spine is derived from the head alone, and a lookup descends in O(log n)
//! bounded header reads.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{PersistedRecordIdentity, RecordAnchorName};

use super::OrderedJournalName;

const MAGIC: [u8; 8] = *b"WSCJREC1";
const FIXED_HEADER_BYTES: usize = 8 + 32 + 8 + 32 + 8 + 8;
const SPINE_ENTRY_BYTES: usize = 8 + 16 + 8;
/// A position is a `u64`, so a spine names at most 64 distinct records.
const MAXIMUM_SPINE_ENTRIES: usize = 64;
pub(super) const MAXIMUM_HEADER_BYTES: usize =
    FIXED_HEADER_BYTES + MAXIMUM_SPINE_ENTRIES * SPINE_ENTRY_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpineEntry {
    pub(super) position: u64,
    pub(super) record: PersistedRecordIdentity,
}

/// The decoded header of one journal record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct JournalRecordHeader {
    pub(super) journal: OrderedJournalName,
    pub(super) position: u64,
    pub(super) payload_sha256: [u8; 32],
    pub(super) payload_bytes: u64,
    /// Strictly descending positions, each below `position`.
    pub(super) spine: Vec<SpineEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JournalFrameDenial {
    Truncated,
    WrongMagic,
    WrongJournal,
    Malformed,
    PayloadDigestMismatch,
}

impl JournalRecordHeader {
    pub(super) fn encoded_bytes(&self) -> usize {
        FIXED_HEADER_BYTES + self.spine.len() * SPINE_ENTRY_BYTES
    }

    /// Encodes the full record: header followed by `payload`.
    pub(super) fn encode_record(&self, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_bytes() + payload.len());
        bytes.extend_from_slice(&MAGIC);
        bytes.extend_from_slice(self.journal.as_str().as_bytes());
        bytes.resize(8 + 32, 0);
        bytes.extend_from_slice(&self.position.to_le_bytes());
        bytes.extend_from_slice(&self.payload_sha256);
        bytes.extend_from_slice(&self.payload_bytes.to_le_bytes());
        bytes.push(self.spine.len() as u8);
        bytes.extend_from_slice(&[0; 7]);
        for entry in &self.spine {
            bytes.extend_from_slice(&entry.position.to_le_bytes());
            bytes.extend_from_slice(&entry.record.allocation_epoch());
            bytes.extend_from_slice(&entry.record.ordinal().to_le_bytes());
        }
        bytes.extend_from_slice(payload);
        bytes
    }

    /// Decodes a header from the leading bytes of a record. With `record_bytes`
    /// the header must also account for the record's exact length.
    pub(super) fn decode(
        bytes: &[u8],
        journal: OrderedJournalName,
        record_bytes: Option<u64>,
    ) -> Result<Self, JournalFrameDenial> {
        if bytes.len() < FIXED_HEADER_BYTES {
            return Err(JournalFrameDenial::Truncated);
        }
        if bytes[..8] != MAGIC {
            return Err(JournalFrameDenial::WrongMagic);
        }
        let mut name = [0; 32];
        name.copy_from_slice(&bytes[8..40]);
        if RecordAnchorName::new(trimmed(&name)?) != Some(journal.anchor()) {
            return Err(JournalFrameDenial::WrongJournal);
        }
        let position = u64::from_le_bytes(bytes[40..48].try_into().expect("fixed field"));
        let payload_sha256 = bytes[48..80].try_into().expect("fixed field");
        let payload_bytes = u64::from_le_bytes(bytes[80..88].try_into().expect("fixed field"));
        let count = usize::from(bytes[88]);
        if position == 0 || count > MAXIMUM_SPINE_ENTRIES || bytes[89..96] != [0; 7] {
            return Err(JournalFrameDenial::Malformed);
        }
        let header_bytes = FIXED_HEADER_BYTES + count * SPINE_ENTRY_BYTES;
        if bytes.len() < header_bytes {
            return Err(JournalFrameDenial::Truncated);
        }
        let total = (header_bytes as u64).checked_add(payload_bytes);
        if total.is_none() || record_bytes.is_some_and(|bytes| total != Some(bytes)) {
            return Err(JournalFrameDenial::Malformed);
        }
        let mut spine = Vec::with_capacity(count);
        for index in 0..count {
            let entry =
                &bytes[FIXED_HEADER_BYTES + index * SPINE_ENTRY_BYTES..][..SPINE_ENTRY_BYTES];
            let entry_position = u64::from_le_bytes(entry[..8].try_into().expect("fixed field"));
            let record = PersistedRecordIdentity::new(
                entry[8..24].try_into().expect("fixed field"),
                u64::from_le_bytes(entry[24..32].try_into().expect("fixed field")),
            )
            .ok_or(JournalFrameDenial::Malformed)?;
            let descending = spine
                .last()
                .map_or(entry_position < position, |prior: &SpineEntry| {
                    entry_position < prior.position
                });
            if entry_position == 0 || !descending {
                return Err(JournalFrameDenial::Malformed);
            }
            spine.push(SpineEntry {
                position: entry_position,
                record,
            });
        }
        if spine != required_spine(position, &spine).ok_or(JournalFrameDenial::Malformed)? {
            return Err(JournalFrameDenial::Malformed);
        }
        Ok(Self {
            journal,
            position,
            payload_sha256,
            payload_bytes,
            spine,
        })
    }

    /// The header of the record that follows this head at `position + 1`.
    pub(super) fn successor(
        journal: OrderedJournalName,
        head: Option<(&Self, PersistedRecordIdentity)>,
        payload: &[u8],
    ) -> Option<Self> {
        let (position, spine) = match head {
            None => (1, Vec::new()),
            Some((head, record)) => {
                let mut candidates = vec![SpineEntry {
                    position: head.position,
                    record,
                }];
                candidates.extend_from_slice(&head.spine);
                let position = head.position.checked_add(1)?;
                (position, required_spine(position, &candidates)?)
            }
        };
        Some(Self {
            journal,
            position,
            payload_sha256: Sha256::digest(payload).into(),
            payload_bytes: payload.len() as u64,
            spine,
        })
    }

    /// The spine entry to visit next on the way down to `target`: the
    /// smallest-positioned entry at or above it.
    pub(super) fn descend_toward(&self, target: u64) -> Option<SpineEntry> {
        self.spine
            .iter()
            .rev()
            .find(|entry| entry.position >= target)
            .copied()
    }

    pub(super) fn verify_payload(&self, payload: &[u8]) -> Result<(), JournalFrameDenial> {
        if payload.len() as u64 != self.payload_bytes
            || <[u8; 32]>::from(Sha256::digest(payload)) != self.payload_sha256
        {
            return Err(JournalFrameDenial::PayloadDigestMismatch);
        }
        Ok(())
    }
}

/// The spine a record at `position` must carry, drawn from `candidates`.
/// `None` when a required earlier record is not among them.
fn required_spine(position: u64, candidates: &[SpineEntry]) -> Option<Vec<SpineEntry>> {
    let below = position.checked_sub(1)?;
    let mut spine: Vec<SpineEntry> = Vec::new();
    for shift in 0..64 {
        let target = (below >> shift) << shift;
        if target == 0 {
            break;
        }
        if spine.last().is_some_and(|entry| entry.position == target) {
            continue;
        }
        let entry = candidates.iter().find(|entry| entry.position == target)?;
        spine.push(*entry);
    }
    Some(spine)
}

fn trimmed(name: &[u8; 32]) -> Result<&str, JournalFrameDenial> {
    let end = name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name.len());
    if name[end..].iter().any(|byte| *byte != 0) {
        return Err(JournalFrameDenial::WrongJournal);
    }
    std::str::from_utf8(&name[..end]).map_err(|_| JournalFrameDenial::WrongJournal)
}

#[cfg(test)]
#[path = "frame/tests.rs"]
mod tests;
