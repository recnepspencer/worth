//! Root-referenced record anchors: a fixed name bound to the record that is
//! currently authoritative for it and to a strictly sequential generation.
//!
//! An anchor is authoritative durable state. It advances only inside the root
//! publication of the batch that wrote its record, and only from the exact
//! generation the publisher observed.

use crate::PersistedRecordIdentity;

use super::durable_root_routing::{decode_identity, encode_identity};

pub const MAXIMUM_RECORD_ANCHORS: usize = 4;
pub const RECORD_ANCHOR_NAME_BYTES: usize = 32;
pub(super) const RECORD_ANCHOR_ENTRY_BYTES: usize = RECORD_ANCHOR_NAME_BYTES + 24 + 32 + 8;
pub(super) const RECORD_ANCHOR_TABLE_BYTES: usize =
    8 + MAXIMUM_RECORD_ANCHORS * RECORD_ANCHOR_ENTRY_BYTES;

/// A declared anchor name: 1 to 32 bytes of `[a-z0-9.-]`, zero padded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordAnchorName([u8; RECORD_ANCHOR_NAME_BYTES]);

impl RecordAnchorName {
    pub fn new(name: &str) -> Option<Self> {
        let bytes = name.as_bytes();
        if bytes.is_empty()
            || bytes.len() > RECORD_ANCHOR_NAME_BYTES
            || !bytes
                .iter()
                .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-'))
        {
            return None;
        }
        let mut padded = [0; RECORD_ANCHOR_NAME_BYTES];
        padded[..bytes.len()].copy_from_slice(bytes);
        Some(Self(padded))
    }

    pub fn as_str(&self) -> &str {
        let end = self
            .0
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.0.len());
        std::str::from_utf8(&self.0[..end]).expect("anchor names are ASCII")
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        if bytes[end..].iter().any(|byte| *byte != 0) {
            return None;
        }
        let name = Self::new(std::str::from_utf8(&bytes[..end]).ok()?)?;
        (name.0 == bytes).then_some(name)
    }
}

/// The record currently selected for one anchor name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordAnchor {
    name: RecordAnchorName,
    record: PersistedRecordIdentity,
    record_payload_sha256: [u8; 32],
    generation: u64,
}

impl RecordAnchor {
    pub const fn name(self) -> RecordAnchorName {
        self.name
    }
    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn record_payload_sha256(self) -> [u8; 32] {
        self.record_payload_sha256
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// One requested anchor movement: from `expected_generation` (zero when the
/// anchor is absent) to the next generation, selecting `record`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordAnchorAdvance {
    name: RecordAnchorName,
    expected_generation: u64,
    record: PersistedRecordIdentity,
    record_payload_sha256: [u8; 32],
}

impl RecordAnchorAdvance {
    pub fn new(
        name: RecordAnchorName,
        expected_generation: u64,
        record: PersistedRecordIdentity,
        record_payload_sha256: [u8; 32],
    ) -> Option<Self> {
        (expected_generation < u64::MAX).then_some(Self {
            name,
            expected_generation,
            record,
            record_payload_sha256,
        })
    }
    pub const fn name(self) -> RecordAnchorName {
        self.name
    }
    pub const fn expected_generation(self) -> u64 {
        self.expected_generation
    }
    pub const fn next_generation(self) -> u64 {
        self.expected_generation + 1
    }
    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn record_payload_sha256(self) -> [u8; 32] {
        self.record_payload_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordAnchorAdvanceDenial {
    /// The anchor's current generation is not the one the publisher observed.
    GenerationConflict { current: u64, expected: u64 },
    /// A new anchor name would exceed the table bound.
    TableFull,
}

/// The anchors selected by one root, sorted by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecordAnchorTable {
    entries: [Option<RecordAnchor>; MAXIMUM_RECORD_ANCHORS],
}

impl RecordAnchorTable {
    pub const EMPTY: Self = Self {
        entries: [None; MAXIMUM_RECORD_ANCHORS],
    };

    pub const fn is_empty(&self) -> bool {
        self.entries[0].is_none()
    }

    pub fn iter(&self) -> impl Iterator<Item = RecordAnchor> + '_ {
        self.entries.iter().map_while(|entry| *entry)
    }

    pub fn get(&self, name: RecordAnchorName) -> Option<RecordAnchor> {
        self.iter().find(|anchor| anchor.name == name)
    }

    pub fn generation(&self, name: RecordAnchorName) -> u64 {
        self.get(name).map_or(0, RecordAnchor::generation)
    }

    pub fn advanced(self, advance: RecordAnchorAdvance) -> Result<Self, RecordAnchorAdvanceDenial> {
        let current = self.generation(advance.name);
        if current != advance.expected_generation {
            return Err(RecordAnchorAdvanceDenial::GenerationConflict {
                current,
                expected: advance.expected_generation,
            });
        }
        let next = RecordAnchor {
            name: advance.name,
            record: advance.record,
            record_payload_sha256: advance.record_payload_sha256,
            generation: advance.next_generation(),
        };
        let mut anchors: Vec<RecordAnchor> = self.iter().filter(|a| a.name != next.name).collect();
        if anchors.len() == MAXIMUM_RECORD_ANCHORS {
            return Err(RecordAnchorAdvanceDenial::TableFull);
        }
        anchors.push(next);
        anchors.sort_by_key(|anchor| anchor.name);
        let mut table = Self::EMPTY;
        for (slot, anchor) in table.entries.iter_mut().zip(anchors) {
            *slot = Some(anchor);
        }
        Ok(table)
    }

    pub(super) fn encode(&self, target: &mut [u8]) {
        let count = self.iter().count();
        target[0] = count as u8;
        for (index, anchor) in self.iter().enumerate() {
            let entry =
                &mut target[8 + index * RECORD_ANCHOR_ENTRY_BYTES..][..RECORD_ANCHOR_ENTRY_BYTES];
            entry[..32].copy_from_slice(&anchor.name.0);
            encode_identity(&mut entry[32..56], anchor.record);
            entry[56..88].copy_from_slice(&anchor.record_payload_sha256);
            entry[88..96].copy_from_slice(&anchor.generation.to_le_bytes());
        }
    }

    /// Decodes a non-empty, canonically sorted table with zeroed unused space.
    pub(super) fn decode(bytes: &[u8]) -> Option<Self> {
        let count = usize::from(bytes[0]);
        if count == 0 || count > MAXIMUM_RECORD_ANCHORS || bytes[1..8] != [0; 7] {
            return None;
        }
        let mut table = Self::EMPTY;
        for index in 0..MAXIMUM_RECORD_ANCHORS {
            let entry =
                &bytes[8 + index * RECORD_ANCHOR_ENTRY_BYTES..][..RECORD_ANCHOR_ENTRY_BYTES];
            if index >= count {
                if entry.iter().any(|byte| *byte != 0) {
                    return None;
                }
                continue;
            }
            let anchor = RecordAnchor {
                name: RecordAnchorName::decode(&entry[..32])?,
                record: decode_identity(&entry[32..56])?,
                record_payload_sha256: entry[56..88].try_into().ok()?,
                generation: u64::from_le_bytes(entry[88..96].try_into().ok()?),
            };
            if anchor.generation == 0
                || index > 0
                    && table.entries[index - 1].is_some_and(|prior| prior.name >= anchor.name)
            {
                return None;
            }
            table.entries[index] = Some(anchor);
        }
        Some(table)
    }
}

#[cfg(test)]
#[path = "record_anchor/tests.rs"]
mod tests;
