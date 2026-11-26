//! `CHK24` paged record directories (`G7`).
//!
//! A region leaf's kind-prefixed record membership is stored as two immutable
//! content-addressed paged directories (entities, relations). The manifest
//! retains only their fixed-size roots, each domain's membership commitment
//! and present kinds, and the page roster that checkpoint custody pins.
//! Admission and validation read no page; cold reads fault only the pages a
//! lookup or kind scan touches through the home's working set.
use serde::{Deserialize, Serialize};

use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::KindId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_io::ChunkMemberSink;
use crate::storage::residency::chunked::paged_directory::{
    DirectoryBuildError, PagedDirectoryBuilder, PagedDirectoryRoot,
};

/// `(kind, slot, generation)`: orders exactly like the membership's
/// kind-prefixed `u128` key, so one kind's members are one contiguous range.
pub type RecordDirectoryKey = (u32, u64, u32);

/// Entries per record directory page and the bound on one page member.
pub const RECORD_DIRECTORY_PAGE_ENTRIES: usize = 256;
pub const RECORD_DIRECTORY_PAGE_BYTES: usize = 64 * 1024;

pub const fn directory_key(encoded: u128) -> RecordDirectoryKey {
    (
        (encoded >> 96) as u32,
        (encoded >> 32) as u64,
        encoded as u32,
    )
}

pub const fn encoded_key((kind, slot, generation): RecordDirectoryKey) -> u128 {
    ((kind as u128) << 96) | ((slot as u128) << 32) | generation as u128
}

/// One record domain's paged membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedRecordMembership {
    pub root: Option<PagedDirectoryRoot>,
    /// The membership commitment the region tree commits to.
    pub digest: [u8; 32],
    /// Kinds with at least one member: bounded by the schema, not by records.
    pub kinds: Vec<KindId>,
}

impl PagedRecordMembership {
    /// The membership of an empty domain.
    pub fn empty() -> Self {
        Self {
            root: None,
            digest: empty_membership_digest(),
            kinds: Vec::new(),
        }
    }
}

/// The record directory facts a `CHK24` region leaf retains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedRecordDirectory {
    pub entities: PagedRecordMembership,
    pub relations: PagedRecordMembership,
    /// Every page of both directories, sorted and deduplicated.
    pub pages: Vec<RelationalDurableMemberLocator>,
}

/// Records every page locator a directory build writes.
struct Roster<'sink> {
    inner: &'sink mut dyn ChunkMemberSink,
    pages: Vec<RelationalDurableMemberLocator>,
}

impl ChunkMemberSink for Roster<'_> {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        self.pages.push(locator);
        self.inner.put(locator, bytes)
    }
}

/// One domain's sorted, unique membership keys and commitment.
pub struct MembershipImage<'a> {
    pub keys: &'a [u128],
    pub digest: [u8; 32],
}

/// Writes both domains as paged directories into `sink`.
pub fn encode_record_directory(
    entities: MembershipImage<'_>,
    relations: MembershipImage<'_>,
    sink: &mut dyn ChunkMemberSink,
) -> Result<PagedRecordDirectory, Cause> {
    let mut roster = Roster {
        inner: sink,
        pages: Vec::new(),
    };
    let entities = encode_membership(&mut roster, entities)?;
    let relations = encode_membership(&mut roster, relations)?;
    let mut pages = roster.pages;
    pages.sort_unstable();
    pages.dedup();
    Ok(PagedRecordDirectory {
        entities,
        relations,
        pages,
    })
}

fn encode_membership(
    sink: &mut dyn ChunkMemberSink,
    image: MembershipImage<'_>,
) -> Result<PagedRecordMembership, Cause> {
    let mut builder =
        PagedDirectoryBuilder::new(RECORD_DIRECTORY_PAGE_ENTRIES, RECORD_DIRECTORY_PAGE_BYTES)
            .map_err(build_cause)?;
    let mut kinds: Vec<KindId> = Vec::new();
    for &encoded in image.keys {
        let key = directory_key(encoded);
        if kinds.last().is_none_or(|last| last.0 != key.0) {
            kinds.push(KindId(key.0));
        }
        builder.push(sink, key, ()).map_err(build_cause)?;
    }
    Ok(PagedRecordMembership {
        root: builder.finish(sink).map_err(build_cause)?,
        digest: image.digest,
        kinds,
    })
}

fn build_cause(error: DirectoryBuildError) -> Cause {
    match error {
        DirectoryBuildError::Member(cause) => cause,
        // Unordered or duplicate membership is a damaged directory image.
        DirectoryBuildError::KeyOutOfOrder | DirectoryBuildError::PageTooSmall => Cause::Damaged,
    }
}

/// The leaf facts a paged directory must agree with.
#[derive(Debug, Clone, Copy)]
pub struct RecordDirectoryLeafFacts {
    pub live_entities: usize,
    pub live_relations: usize,
    pub entity_slots: usize,
    pub relation_slots: usize,
}

/// The commitment of an empty membership.
pub fn empty_membership_digest() -> [u8; 32] {
    crate::storage::substrate::DigestIndex::default().digest()
}

impl PagedRecordDirectory {
    /// Read-free structural admission: a sorted, unique, size-bounded roster
    /// holding every root; non-empty roots whose counts agree with the leaf's
    /// live counts and slot domains; and kinds that are sorted, unique and
    /// present exactly when their domain is.
    pub fn validate(&self, facts: RecordDirectoryLeafFacts) -> Result<(), &'static str> {
        let limit = RECORD_DIRECTORY_PAGE_BYTES as u64;
        if self.pages.windows(2).any(|pair| pair[0] >= pair[1])
            || self.pages.iter().any(|page| page.length > limit)
        {
            return Err("record directory page roster is duplicate, unsorted or oversized");
        }
        let domains = [
            (&self.entities, facts.live_entities, facts.entity_slots),
            (&self.relations, facts.live_relations, facts.relation_slots),
        ];
        let empty_digest = empty_membership_digest();
        let mut total = 0_u64;
        for (membership, live, slots) in domains {
            let count = membership.validate(&self.pages, live, slots, empty_digest)?;
            total = total
                .checked_add(count)
                .ok_or("record directory entry count overflow")?;
        }
        // Every page holds at least one entry or two children.
        let page_bound = total
            .checked_mul(2)
            .ok_or("record directory entry count overflow")?;
        if self.pages.is_empty() != (total == 0) || self.pages.len() as u64 > page_bound {
            return Err("record directory page roster disagrees with its roots");
        }
        Ok(())
    }
}

impl PagedRecordMembership {
    fn validate(
        &self,
        pages: &[RelationalDurableMemberLocator],
        live: usize,
        slots: usize,
        empty_digest: [u8; 32],
    ) -> Result<u64, &'static str> {
        if self.kinds.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("record directory kinds are duplicate or unsorted");
        }
        let Some(root) = self.root else {
            if live != 0 || !self.kinds.is_empty() || self.digest != empty_digest {
                return Err("empty record directory disagrees with its leaf");
            }
            return Ok(0);
        };
        if root.entry_count == 0 || pages.binary_search(&root.root_page).is_err() {
            return Err("record directory root is empty or absent from page roster");
        }
        if usize::try_from(root.entry_count).ok() != Some(live)
            || root.entry_count > slots as u64
            || self.kinds.is_empty()
            || self.kinds.len() as u64 > root.entry_count
        {
            return Err("record directory root disagrees with its leaf domain");
        }
        Ok(root.entry_count)
    }
}
