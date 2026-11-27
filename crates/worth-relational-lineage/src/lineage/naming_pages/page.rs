//! One authenticated naming page and its check against its descriptor.
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    decode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::page_cache::ResidentPage;

use super::{NamingPageRef, NAMING_KEY_BYTES, NAMING_PAGE_ENTRIES};

/// The encoded page: strictly ascending `(row key, value)` rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingPage {
    pub rows: Vec<(Vec<u8>, Vec<u8>)>,
}

/// A decoded resident page and the encoded size that bounds its heap.
#[derive(Debug)]
pub struct ResidentNamingPage {
    pub page: NamingPage,
    encoded: u64,
    continuation_chunks: u32,
    continuation_digest: Option<[u8; 32]>,
}

impl NamingPage {
    fn is_canonical(&self) -> bool {
        !self.rows.is_empty()
            && self.rows.len() <= NAMING_PAGE_ENTRIES as usize
            && self
                .rows
                .iter()
                .all(|(key, _)| !key.is_empty() && key.len() <= NAMING_KEY_BYTES)
            && self.rows.windows(2).all(|pair| pair[0].0 < pair[1].0)
    }
}

impl ResidentPage for ResidentNamingPage {
    fn decode(bytes: &[u8], locator: RelationalDurableMemberLocator) -> Result<Self, Cause> {
        let page: NamingPage = decode_member(ChunkMemberKind::NamingStatePage, bytes, locator)?;
        if !page.is_canonical() {
            return Err(Cause::Damaged);
        }
        super::tables::verify_group_log_single_branch(&page.rows)?;
        let (continuation_chunks, continuation_digest) =
            super::tables::page_continuation_summary(&page.rows)?;
        Ok(Self {
            page,
            encoded: bytes.len() as u64,
            continuation_chunks,
            continuation_digest,
        })
    }

    fn resident_bytes(&self) -> u64 {
        let rows = self.page.rows.capacity() * std::mem::size_of::<(Vec<u8>, Vec<u8>)>();
        (std::mem::size_of::<Self>() + rows) as u64 + self.encoded
    }
}

impl ResidentNamingPage {
    /// Checks a faulted page against its descriptor in O(1) using cached
    /// decode-time continuation metadata (zero per-hit row scanning).
    pub fn admit(&self, reference: &NamingPageRef) -> Result<(), Cause> {
        let rows = &self.page.rows;
        if rows.len() != reference.entries as usize
            || rows.first().map(|row| &row.0) != Some(&reference.first)
            || rows.last().map(|row| &row.0) != Some(&reference.last)
            || self.continuation_chunks != reference.continuation_chunks
            || self.continuation_digest != reference.continuation_digest
        {
            return Err(Cause::IntegrityMismatch);
        }
        Ok(())
    }

    /// The value of `key`.
    ///
    /// Complexity: O(log rows).
    pub fn value(&self, key: &[u8]) -> Option<&[u8]> {
        self.page
            .rows
            .binary_search_by(|(row, _)| row.as_slice().cmp(key))
            .ok()
            .map(|slot| self.page.rows[slot].1.as_slice())
    }
}
