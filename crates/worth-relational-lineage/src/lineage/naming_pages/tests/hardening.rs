//! Hardening unit proofs for Store M4.3f Round 1 review fixes:
//! - `[P1-4]` > 32 KiB values (`NameEntry` + `GROUP_LOG`) chunk transparently
//!   across `<= 64 KiB` naming pages and clean up stale chunks on shrink/delete.
//! - `[P1-6]` Cold `advance_retention_floor` is atomic under mid-pass damage.
//! - `[P1-8]` Layered `TieredMap` fork + write avoids deep-cloning shared delta.
//! - `[P2-11]` Owner-bound non-`Copy` `FullLineageRecoveryToken` rejects
//!   cross-state reuse.
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GroupEvent;
use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_contracts::internal_port::lineage::data::name_resolution::PersistentNameKey;
use worth_relational_contracts::internal_port::lineage::data::naming::{
    PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
};
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;

use super::MemSource;
use crate::lineage::index::{
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameIndexDenial,
};
use crate::lineage::naming_pages::builder::seal_pages;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::{row_key, GROUP_LOG, INDEX_MEMBER_GROUPS, INDEX_NAME};
use crate::lineage::naming_pages::{BranchNamingRoot, CheckpointNamingState, NAMING_PAGE_BYTES};
use crate::lineage::tiered::{ColdNamingSource, NamingBaseLink, PinnedNamingStore, TieredMap};
use crate::runtime::state::subsystems::lineage::LineageState;

#[derive(Debug)]
struct StaticSource(Arc<ColdNamingStore>);

impl ColdNamingSource for StaticSource {
    fn store(&self) -> Option<PinnedNamingStore> {
        Some(PinnedNamingStore::unpinned(Arc::clone(&self.0)))
    }
}

fn branch_state(
    branch: BranchId,
    pages: Vec<crate::lineage::naming_pages::NamingPageRef>,
    group_log: Vec<crate::lineage::naming_pages::NamingPageRef>,
    group_log_through: Option<u64>,
) -> Arc<CheckpointNamingState> {
    let root = BranchNamingRoot {
        branch,
        naming: None,
        index: None,
        groups: None,
        pages,
        delta_base: None,
    };
    Arc::new(CheckpointNamingState {
        branches: vec![root],
        group_log,
        group_log_through,
        retired_forks: Vec::new(),
    })
}

#[test]
fn chunked_large_values_above_32kib_roundtrip_and_shrink_cleanly() {
    let huge_name_key = row_key(
        INDEX_NAME,
        &PersistentNameKey::Group(AggregateIdentity::from_bytes([0x55; 32])),
    );
    let huge_name_val: Vec<u8> = (0..75_000usize).map(|i| (i * 31 % 251) as u8).collect();
    let small_key = row_key(
        INDEX_NAME,
        &PersistentNameKey::Group(AggregateIdentity::from_bytes([0x01; 32])),
    );
    let small_val = vec![0x99u8; 64];
    let mut rows = vec![
        (small_key.clone(), Some(small_val.clone())),
        (huge_name_key.clone(), Some(huge_name_val.clone())),
    ];
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sink = CollectedMembers::default();
    let branch_pages = seal_pages(None, rows, &mut sink).unwrap();
    assert!(branch_pages
        .iter()
        .all(|p| p.member.length <= NAMING_PAGE_BYTES as u64));

    let main_digest =
        crate::lineage::naming_pages::tables::branch_key_digest(&BranchId("main".into()));
    let log_key = row_key(GROUP_LOG, &((1u64, CommitId(7)), main_digest));
    let huge_log_frame: Vec<u8> = (0..68_000usize).map(|i| (i * 17 % 239) as u8).collect();
    let log_pages = seal_pages(
        None,
        vec![(log_key.clone(), Some(huge_log_frame.clone()))],
        &mut sink,
    )
    .unwrap();
    assert!(log_pages
        .iter()
        .all(|p| p.member.length <= NAMING_PAGE_BYTES as u64));

    let state = branch_state(
        BranchId("main".into()),
        branch_pages.clone(),
        log_pages.clone(),
        Some(7),
    );
    assert!(state.validate().is_ok());
    let store = ColdNamingStore::open(Arc::clone(&state), MemSource::from_sink(sink));
    assert_eq!(
        store.get(&branch_pages, &small_key).unwrap(),
        Some(small_val)
    );
    assert_eq!(
        store.get(&branch_pages, &huge_name_key).unwrap(),
        Some(huge_name_val.clone())
    );
    assert_eq!(
        store.get(&log_pages, &log_key).unwrap(),
        Some(huge_log_frame)
    );

    let mut scanned = BTreeMap::new();
    store
        .scan(&branch_pages, &[INDEX_NAME], |k, v| {
            scanned.insert(k.to_vec(), v.to_vec());
            Ok(true)
        })
        .unwrap();
    assert_eq!(scanned.len(), 2);
    assert_eq!(scanned.get(&huge_name_key), Some(&huge_name_val));

    // Shrink back to inline and confirm stale continuation chunks are removed.
    let mut sink2 = CollectedMembers::default();
    let shrunk_pages = seal_pages(
        Some((&store, &branch_pages)),
        vec![(huge_name_key.clone(), Some(vec![0xab; 16]))],
        &mut sink2,
    )
    .unwrap();
    let state2 = branch_state(
        BranchId("main".into()),
        shrunk_pages.clone(),
        Vec::new(),
        None,
    );
    assert!(state2.validate().is_ok());
    let store2 = ColdNamingStore::open(state2, MemSource::from_sink(sink2));
    assert_eq!(
        store2.get(&shrunk_pages, &huge_name_key).unwrap(),
        Some(vec![0xab; 16])
    );
    assert!(shrunk_pages
        .iter()
        .all(|p| p.last[0] != crate::lineage::naming_pages::tables::VALUE_CHUNK));
}

#[test]
fn cold_advance_retention_floor_is_atomic_on_mid_pass_damage() {
    let branch = BranchId("main".into());
    let mut resident = PersistentNameBranchIndex::new(branch.clone(), None);
    let g = AggregateIdentity::from_bytes([0x33; 32]);
    let gk = PersistentNameKey::Group(g);
    let m1 = PersistentNameIdentity::from_minted_digest([0x11; 32]);
    let m2 = PersistentNameIdentity::from_minted_digest([0x22; 32]);
    let mut birth = PersistentNameCommittedBatch::new(CommitId(1))
        .with_group_events([GroupEvent::Born { group: g }])
        .with_carriers(gk, [m1, m2]);
    birth.group_schemes.insert(g, "s1".into());
    resident.apply(&birth).unwrap();
    for c in 2..=12u64 {
        let carriers = if c % 2 == 0 { vec![m1] } else { vec![m1, m2] };
        resident
            .apply(&PersistentNameCommittedBatch::new(CommitId(c)).with_carriers(gk, carriers))
            .unwrap();
    }
    // Pad enough distinct member keys so `INDEX_MEMBER_GROUPS` lives on its own page.
    for i in 0..300u16 {
        let mut digest = [0u8; 32];
        digest[0..2].copy_from_slice(&i.to_be_bytes());
        digest[31] = 0x77;
        let mi = PersistentNameIdentity::from_minted_digest(digest);
        resident
            .apply(
                &PersistentNameCommittedBatch::new(CommitId(20 + i as u64))
                    .with_carriers(gk, [m1, mi]),
            )
            .unwrap();
    }
    let mut changes = resident.encoded_changes();
    changes.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut sink = CollectedMembers::default();
    let pages = seal_pages(None, changes, &mut sink).unwrap();
    let member_page = pages
        .iter()
        .find(|p| p.first[0] == INDEX_MEMBER_GROUPS)
        .expect("dedicated member_groups page");
    let source = MemSource::from_sink(sink);
    {
        let mut guard = source.members.lock().unwrap();
        let bytes = guard.get_mut(&member_page.member).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
    }
    let state = branch_state(branch.clone(), pages, Vec::new(), None);
    let store = Arc::new(ColdNamingStore::open(state, source));
    let link = NamingBaseLink::new(Arc::new(StaticSource(store)), branch.clone());
    let mut cold_idx = PersistentNameBranchIndex::over(
        branch,
        None,
        resident.earliest_commit(),
        resident.head(),
        Some(link),
    );
    let err = cold_idx.advance_retention_floor(CommitId(100)).unwrap_err();
    assert_eq!(
        err,
        PersistentNameIndexDenial::StorageUnavailable(Cause::IntegrityMismatch)
    );
    assert_eq!(cold_idx.retention_floor(), None);
    assert!(!cold_idx.names.has_delta());
}

#[test]
fn layered_tiered_map_fork_promotes_without_cloning_and_preserves_isolation() {
    let mut parent: TieredMap<CommitId, u64> = TieredMap::resident(INDEX_NAME);
    for i in 1..=64u64 {
        parent.insert(CommitId(i), i * 10);
    }
    let mut child = parent.fork(None);
    child.insert(CommitId(1), 999);
    child.remove_blind(&CommitId(2));
    assert_eq!(parent.get(&CommitId(1)).unwrap().as_deref(), Some(&10));
    assert_eq!(parent.get(&CommitId(2)).unwrap().as_deref(), Some(&20));
    assert_eq!(child.get(&CommitId(1)).unwrap().as_deref(), Some(&999));
    assert_eq!(child.get(&CommitId(2)).unwrap().as_deref(), None);
    assert_eq!(child.get(&CommitId(64)).unwrap().as_deref(), Some(&640));

    // Mutating or removing an already-present key right after `fork` (when
    // `frozen` is empty and the key lives only in the shared `active` layer)
    // must promote first and write a tombstone without disturbing `parent`.
    let mut fork_mut = parent.fork(None);
    *fork_mut
        .get_mut(&CommitId(1))
        .unwrap()
        .expect("forked key must be mutable via get_mut") = 1000;
    assert_eq!(fork_mut.get(&CommitId(1)).unwrap().as_deref(), Some(&1000));
    assert_eq!(parent.get(&CommitId(1)).unwrap().as_deref(), Some(&10));
    let mut fork_del = parent.fork(None);
    assert_eq!(fork_del.remove(&CommitId(3)).unwrap(), Some(30));
    assert_eq!(fork_del.get(&CommitId(3)).unwrap(), None);
    assert_eq!(parent.get(&CommitId(3)).unwrap().as_deref(), Some(&30));
}

#[test]
#[should_panic(expected = "different LineageState")]
fn full_lineage_recovery_token_rejects_foreign_state() {
    let state_a = LineageState::default();
    let mut state_b = LineageState::default();
    let token_a = state_a.full_history_token().expect("fresh state has token");
    state_b.rebuild_name_indexes(token_a);
}

#[test]
fn frozen_capture_pins_store_and_custody_across_concurrent_source_swap() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use crate::runtime::state::subsystems::lineage::LineageSubsystem;

    struct CustodyFlag(Arc<AtomicBool>);
    impl Drop for CustodyFlag {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[derive(Debug)]
    struct SwappableSource {
        slot: Mutex<Option<PinnedNamingStore>>,
    }
    impl ColdNamingSource for SwappableSource {
        fn store(&self) -> Option<PinnedNamingStore> {
            self.slot.lock().unwrap().clone()
        }
    }

    let branch = BranchId("main".into());
    let g = AggregateIdentity::from_bytes([0x9a; 32]);
    let gk = PersistentNameKey::Group(g);
    let m1 = PersistentNameIdentity::from_minted_digest([0x41; 32]);
    let mut idx = PersistentNameBranchIndex::new(branch.clone(), None);
    let mut birth = PersistentNameCommittedBatch::new(CommitId(1))
        .with_group_events([GroupEvent::Born { group: g }])
        .with_carriers(gk, [m1]);
    birth.group_schemes.insert(g, "s1".into());
    idx.apply(&birth).unwrap();

    let mut changes = idx.encoded_changes();
    changes.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut sink1 = CollectedMembers::default();
    let pages1 = seal_pages(None, changes, &mut sink1).unwrap();
    let hdr = crate::lineage::naming_pages::NameIndexHeader {
        earliest_commit: Some(1),
        head: Some(1),
        retention_floor: None,
        unavailable: false,
    };
    let mut s1 = (*branch_state(branch.clone(), pages1.clone(), Vec::new(), None)).clone();
    s1.branches[0].index = Some(hdr);
    let state1 = Arc::new(s1);
    let dropped_c1 = Arc::new(AtomicBool::new(false));
    let store1 = Arc::new(ColdNamingStore::open(
        Arc::clone(&state1),
        MemSource::from_sink(sink1),
    ));
    let pinned1 = PinnedNamingStore::new(store1, Arc::new(CustodyFlag(Arc::clone(&dropped_c1))));
    let swappable = Arc::new(SwappableSource {
        slot: Mutex::new(Some(pinned1)),
    });
    let lineage = LineageSubsystem::new();
    lineage.install_cold_naming(swappable.clone(), &state1);
    let frozen = lineage.freeze_naming();
    *swappable.slot.lock().unwrap() = None;
    assert!(!dropped_c1.load(Ordering::SeqCst));
    let mut sink_out = CollectedMembers::default();
    let sealed = frozen.seal(&[branch.clone()], &mut sink_out).unwrap();
    assert_eq!(sealed.root(&branch).unwrap().pages, pages1);
    drop(frozen);
    assert!(dropped_c1.load(Ordering::SeqCst));
}
