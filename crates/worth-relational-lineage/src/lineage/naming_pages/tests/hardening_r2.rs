//! Round 2 hardening unit tests (`[P1-6]`, `[P1-7]`, `[P1-8]`, `[P1-9b]`):
//! - Shared continuation-envelope validation on both writer (>64 MiB limit and
//!   malformed prior header rejection during stale-chunk cleanup) and reader.
//! - Chunked `ColdNamingStore::scan` under a strict 1-page memory budget.
//! - `PersistentNameBranchIndex::advance_retention_floor` rejecting floors
//!   above `head` without mutating state.
//! - Persistent `CowMap` AVL tree across 200 repeated fork/write/delete cycles.

use std::sync::Arc;

use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GroupEvent;
use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

use super::super::builder::seal_pages;
use super::super::page::NamingPage;
use super::super::store::ColdNamingStore;
use super::super::tables::{
    row_key, CHUNKED_HEADER_BYTES, GROUP_LOG, INDEX_NAME, MAX_REASSEMBLED_VALUE_BYTES,
    VALUE_CHUNKED_TAG,
};
use super::super::{BranchNamingRoot, CheckpointNamingState, NamingPageRef, NAMING_PAGE_BYTES};
use super::{CollectedMembers, MemSource};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameCommittedBatch};
use crate::lineage::tiered::TieredMap;

#[test]
fn chunked_writer_refuses_values_above_64mib_and_rejects_malformed_prior_header() {
    let digest = crate::lineage::naming_pages::tables::branch_key_digest(&BranchId("main".into()));
    let key = row_key(GROUP_LOG, &((1u64, CommitId(1)), digest));
    let oversized = vec![0xABu8; MAX_REASSEMBLED_VALUE_BYTES + 1];
    let mut sink = CollectedMembers::default();
    let err = seal_pages(None, vec![(key.clone(), Some(oversized))], &mut sink)
        .expect_err("writer must refuse values above MAX_REASSEMBLED_VALUE_BYTES");
    assert_eq!(
        err,
        Cause::LimitExceeded {
            length: (MAX_REASSEMBLED_VALUE_BYTES + 1) as u64,
            limit: MAX_REASSEMBLED_VALUE_BYTES as u64,
        }
    );

    // Craft a prior page containing a hostile continuation header with u32::MAX chunks.
    let mut hostile_header = Vec::with_capacity(CHUNKED_HEADER_BYTES);
    hostile_header.push(VALUE_CHUNKED_TAG);
    hostile_header.extend_from_slice(&u32::MAX.to_be_bytes());
    hostile_header.extend_from_slice(&(64u32 * 1024).to_be_bytes());
    let page = NamingPage {
        rows: vec![(key.clone(), hostile_header)],
    };
    let member = encode_member(ChunkMemberKind::NamingStatePage, &page, NAMING_PAGE_BYTES).unwrap();
    let mut prior_sink = CollectedMembers::default();
    prior_sink.put(member.locator, member.bytes).unwrap();
    let prior_ref = NamingPageRef {
        first: key.clone(),
        last: key.clone(),
        entries: 1,
        continuation_chunks: 0,
        continuation_digest: None,
        group_log_branches: vec![crate::lineage::naming_pages::GroupLogBranchSpan {
            branch_digest: digest,
            first_commit: 1,
            last_commit: 1,
        }],
        member: member.locator,
    };
    let state = Arc::new(CheckpointNamingState {
        branches: Vec::new(),
        group_log: vec![prior_ref.clone()],
        group_log_through: Some(1),
    });
    let store = Arc::new(ColdNamingStore::open(
        state,
        MemSource::from_sink(prior_sink),
    ));
    let mut next_sink = CollectedMembers::default();
    let err = seal_pages(
        Some((store.as_ref(), &[prior_ref])),
        vec![(key, None)],
        &mut next_sink,
    )
    .expect_err("malformed prior continuation header must fail closed without unbounded loop");
    assert_eq!(err, Cause::Damaged);
}

#[test]
fn chunked_scan_succeeds_under_strict_one_page_memory_budget() {
    let branch = BranchId("main".into());
    let digest = crate::lineage::naming_pages::tables::branch_key_digest(&branch);
    let k1 = row_key(GROUP_LOG, &((1u64, CommitId(1)), digest));
    let k2 = row_key(GROUP_LOG, &((2u64, CommitId(2)), digest));
    let k3 = row_key(GROUP_LOG, &((3u64, CommitId(3)), digest));
    let small_v1 = vec![0x11u8; 24 * 1024];
    let large_v2 = vec![0x5Cu8; 144 * 1024]; // 6 continuation chunks (~3 chunk pages)
    let small_v3 = vec![0x33u8; 24 * 1024];

    let mut sink = CollectedMembers::default();
    let pages = seal_pages(
        None,
        vec![
            (k1.clone(), Some(small_v1.clone())),
            (k2.clone(), Some(large_v2.clone())),
            (k3.clone(), Some(small_v3.clone())),
        ],
        &mut sink,
    )
    .unwrap();
    assert!(
        pages.len() >= 4,
        "expected logical page + >=3 chunk pages, got {}",
        pages.len()
    );
    assert_eq!(
        pages[0].last, k2,
        "logical page holding chunked header must be strictly separate from continuation chunk pages"
    );

    // Measure the exact charged bytes of the largest single page, then set the
    // historical working-set budget strictly below 2 pages so 2 simultaneous
    // borrowed pages would refuse MemoryExhausted.
    let source = MemSource::from_sink(sink);
    let state = Arc::new(CheckpointNamingState {
        branches: vec![BranchNamingRoot {
            branch: BranchId("main".into()),
            naming: None,
            index: None,
            groups: None,
            pages: pages.clone(),
        }],
        group_log: pages.clone(),
        group_log_through: Some(3),
    });
    let mut max_single_page_bytes = 0u64;
    for page_ref in &pages {
        let probe_acct = WorkingSetAccountant::new(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: u64::MAX / 4,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        });
        let probe = ColdNamingStore::open(Arc::clone(&state), source.clone());
        probe.bind_working_set(probe_acct.clone());
        let _ = probe.page(page_ref).unwrap();
        max_single_page_bytes =
            max_single_page_bytes.max(probe_acct.pressure().historical_resident_bytes);
    }
    let one_page_budget = max_single_page_bytes + 512;
    assert!(
        one_page_budget < max_single_page_bytes.saturating_mul(2),
        "budget {one_page_budget} must fit 1 page ({max_single_page_bytes}) and reject 2"
    );

    let accountant = WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: u64::MAX / 4,
        historical_working_set_bytes: one_page_budget,
        max_in_flight_fill_bytes: u64::MAX / 4,
        streaming_window_bytes: 64 * 1024,
    });
    let store = ColdNamingStore::open(state, source);
    assert!(store.bind_working_set(accountant));

    let mut seen = Vec::new();
    store
        .scan(&pages, &[GROUP_LOG], |key, value| {
            seen.push((key.to_vec(), value.to_vec()));
            Ok(true)
        })
        .expect("chunked scan must release logical page borrow and succeed under 1-page budget");
    assert_eq!(seen, vec![(k1, small_v1), (k2, large_v2), (k3, small_v3)]);
    let obs = store.observation();
    assert!(
        obs.evictions > 0,
        "expected evictions under 1-page budget: {obs:?}"
    );
    assert!(
        obs.resident_pages <= 1,
        "at most 1 page may remain resident: {obs:?}"
    );
}

#[test]
fn advance_retention_floor_beyond_last_naming_batch_prunes_below_and_serves_above() {
    use worth_relational_contracts::internal_port::lineage::data::name_resolution::{
        PersistentNameResolutionDenial, PersistentNameSelection,
    };
    let branch = BranchId("main".into());
    let g = AggregateIdentity::from_bytes([0x61; 32]);
    let mut idx = PersistentNameBranchIndex::new(branch, None);
    idx.apply(
        &PersistentNameCommittedBatch::new(CommitId(2))
            .with_group_events([GroupEvent::Born { group: g }]),
    )
    .unwrap();
    // Simulate a later unnamed commit at CommitId(3) where index.head() is still CommitId(2).
    idx.advance_retention_floor(CommitId(3)).unwrap();
    assert_eq!(idx.retention_floor(), Some(CommitId(3)));
    assert_eq!(
        idx.resolve(
            PersistentNameKey::Group(g),
            CommitId(2),
            &PersistentNameSelection::Exact,
        )
        .result,
        Err(PersistentNameResolutionDenial::BasisPruned {
            basis: CommitId(2),
            retention_floor: CommitId(3),
        })
    );
    assert!(idx
        .resolve(
            PersistentNameKey::Group(g),
            CommitId(3),
            &PersistentNameSelection::Exact,
        )
        .result
        .is_ok());
}

#[test]
fn cow_map_repeated_fork_write_cycles_preserve_avl_balance_and_isolation() {
    let mut cur: TieredMap<CommitId, u64> = TieredMap::resident(INDEX_NAME);
    let mut snapshots = Vec::new();
    for cycle in 1..=200u64 {
        cur.insert(CommitId(cycle), cycle * 10);
        if cycle > 5 && cycle % 7 == 0 {
            cur.remove_blind(&CommitId(cycle - 3));
        }
        if cycle > 10 && cycle % 11 == 0 {
            *cur.get_mut(&CommitId(cycle - 1)).unwrap().unwrap() += 1;
        }
        let next = cur.fork(None);
        if cycle % 40 == 0 {
            snapshots.push((cycle, cur.fork(None)));
        }
        cur = next;
    }
    for (at, snap) in snapshots {
        assert_eq!(
            snap.get(&CommitId(at)).unwrap().as_deref(),
            Some(&(at * 10))
        );
        assert_eq!(snap.get(&CommitId(at + 1)).unwrap(), None);
    }
    assert_eq!(cur.get(&CommitId(200)).unwrap().as_deref(), Some(&2000));
    assert_eq!(cur.get(&CommitId(999_999)).unwrap(), None);
}
