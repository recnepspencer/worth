//! Round 3 hardening unit tests (`[P1-7]` and `[P2-9]`):
//! - `[P1-7]` `CowMap` wraps node values in `Arc<V>` so fork + disjoint writes
//!   clone ZERO unrelated payloads, and `get_mut` clones strictly the target key.
//! - `[P2-9]` Oversized inline envelopes (`payload.len() > MAX_INLINE_VALUE_BYTES`)
//!   refuse `Cause::Damaged` consistently across `ColdNamingStore::get` AND both
//!   `ColdNamingStore::scan` paths (inline fast-path and chunked staged path).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use super::super::builder::seal_pages;
use super::super::codec::{NamingCodec, Reader, Writer};
use super::super::page::NamingPage;
use super::super::store::ColdNamingStore;
use super::super::tables::{row_key, INDEX_NAME, MAX_INLINE_VALUE_BYTES, VALUE_INLINE_TAG};
use super::super::{BranchNamingRoot, CheckpointNamingState, NamingPageRef, NAMING_PAGE_BYTES};
use super::{CollectedMembers, MemSource};
use crate::lineage::tiered::TieredMap;

#[derive(Debug)]
struct CountedPayload {
    value: u64,
    clones: Arc<AtomicUsize>,
}

impl PartialEq for CountedPayload {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for CountedPayload {}

impl Clone for CountedPayload {
    fn clone(&self) -> Self {
        self.clones.fetch_add(1, Ordering::SeqCst);
        Self {
            value: self.value,
            clones: Arc::clone(&self.clones),
        }
    }
}

impl NamingCodec for CountedPayload {
    fn write(&self, out: &mut Writer) {
        self.value.write(out);
    }

    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self {
            value: u64::read(input)?,
            clones: Arc::new(AtomicUsize::new(0)),
        })
    }
}

#[test]
fn cow_map_fork_and_disjoint_writes_clone_zero_unrelated_payloads() {
    let clones = Arc::new(AtomicUsize::new(0));
    let payload = |value| CountedPayload {
        value,
        clones: Arc::clone(&clones),
    };
    let mut parent: TieredMap<CommitId, CountedPayload> = TieredMap::resident(INDEX_NAME);
    for i in 1..=64u64 {
        parent.insert(CommitId(i), payload(i * 10));
    }
    assert_eq!(clones.load(Ordering::SeqCst), 0);

    let mut child = parent.fork(None);
    for extra in 100..=115u64 {
        child.insert(CommitId(extra), payload(extra));
    }
    child.insert(CommitId(100), payload(9999));
    child.remove_blind(&CommitId(101));
    child.remove_blind(&CommitId(16));
    assert_eq!(clones.load(Ordering::SeqCst), 0);

    // Mutating one shared key clones strictly that one target payload.
    child.get_mut(&CommitId(32)).unwrap().unwrap().value = 7777;
    assert_eq!(clones.load(Ordering::SeqCst), 1);
    assert_eq!(
        parent.get(&CommitId(32)).unwrap().as_deref().unwrap().value,
        320
    );
    assert_eq!(
        child.get(&CommitId(32)).unwrap().as_deref().unwrap().value,
        7777
    );
    assert_eq!(
        parent.get(&CommitId(16)).unwrap().as_deref().unwrap().value,
        160
    );
    assert_eq!(child.get(&CommitId(16)).unwrap(), None);
}

fn single_branch_store(pages: Vec<NamingPageRef>, sink: CollectedMembers) -> ColdNamingStore {
    let root = BranchNamingRoot {
        branch: BranchId("main".into()),
        naming: None,
        index: None,
        groups: None,
        pages,
    };
    let state = CheckpointNamingState {
        branches: vec![root],
        group_log: Vec::new(),
        group_log_through: None,
    };
    ColdNamingStore::open(Arc::new(state), MemSource::from_sink(sink))
}

#[test]
fn oversized_inline_envelope_refuses_damaged_on_both_get_and_scan() {
    let key = row_key(
        INDEX_NAME,
        &PersistentNameKey::Group(AggregateIdentity::from_bytes([0x44; 32])),
    );
    let mut bad_inline = vec![VALUE_INLINE_TAG];
    bad_inline.resize(1 + MAX_INLINE_VALUE_BYTES + 1, 0xAB);

    let page = NamingPage {
        rows: vec![(key.clone(), bad_inline.clone())],
    };
    let member = encode_member(ChunkMemberKind::NamingStatePage, &page, NAMING_PAGE_BYTES).unwrap();
    let mut sink = CollectedMembers::default();
    sink.put(member.locator, member.bytes).unwrap();
    let page_ref = NamingPageRef {
        first: key.clone(),
        last: key.clone(),
        entries: 1,
        continuation_chunks: 0,
        continuation_digest: None,
        member: member.locator,
    };
    let store = single_branch_store(vec![page_ref.clone()], sink);

    // Inline fast-path: both get() and scan() must refuse Cause::Damaged.
    assert_eq!(store.get(&[page_ref.clone()], &key), Err(Cause::Damaged));
    assert_eq!(
        store.scan(&[page_ref], &[INDEX_NAME], |_, _| Ok(true)),
        Err(Cause::Damaged)
    );

    // Chunked staged scan path: a page mixing a valid chunked row AND an
    // oversized inline row must ALSO refuse Cause::Damaged in scan().
    let k_chunked = row_key(
        INDEX_NAME,
        &PersistentNameKey::Group(AggregateIdentity::from_bytes([0x11; 32])),
    );
    let k_bad = row_key(
        INDEX_NAME,
        &PersistentNameKey::Group(AggregateIdentity::from_bytes([0x22; 32])),
    );
    let mut valid_sink = CollectedMembers::default();
    let valid_pages = seal_pages(
        None,
        vec![(k_chunked.clone(), Some(vec![0x5Au8; 40 * 1024]))],
        &mut valid_sink,
    )
    .unwrap();
    let valid_store = single_branch_store(valid_pages.clone(), valid_sink);
    let chunked_header = valid_store
        .page(&valid_pages[0])
        .unwrap()
        .value(&k_chunked)
        .unwrap()
        .to_vec();
    let mixed_page = NamingPage {
        rows: vec![
            (k_chunked.clone(), chunked_header),
            (k_bad.clone(), bad_inline),
        ],
    };
    let mixed_member = encode_member(
        ChunkMemberKind::NamingStatePage,
        &mixed_page,
        NAMING_PAGE_BYTES,
    )
    .unwrap();
    let mut mixed_sink = CollectedMembers::default();
    mixed_sink
        .put(mixed_member.locator, mixed_member.bytes)
        .unwrap();
    for p in &valid_pages[1..] {
        let raw_page = valid_store.page(p).unwrap();
        let m = encode_member(
            ChunkMemberKind::NamingStatePage,
            &raw_page.page,
            NAMING_PAGE_BYTES,
        )
        .unwrap();
        mixed_sink.put(m.locator, m.bytes).unwrap();
    }
    let mut mixed_refs = vec![NamingPageRef {
        first: k_chunked,
        last: k_bad,
        entries: 2,
        continuation_chunks: valid_pages[0].continuation_chunks,
        continuation_digest: valid_pages[0].continuation_digest,
        member: mixed_member.locator,
    }];
    mixed_refs.extend_from_slice(&valid_pages[1..]);
    let mixed_store = single_branch_store(mixed_refs.clone(), mixed_sink);
    assert_eq!(
        mixed_store.scan(&mixed_refs, &[INDEX_NAME], |_, _| Ok(true)),
        Err(Cause::Damaged)
    );
}
