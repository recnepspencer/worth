//! Unit proofs for `CHK24` paged naming state, working-set eviction,
//! tamper/integrity refusals, and retention-floor compaction kernels.
mod hardening;
mod hardening_r2;
mod hardening_r3;
pub(crate) mod hardening_r4;
mod table_domains;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GroupEvent;
use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_contracts::internal_port::lineage::data::name_resolution::{
    PersistentNameKey, PersistentNameResolutionDenial, PersistentNameSelection,
};
use worth_relational_contracts::internal_port::lineage::data::naming::{
    PersistentNameIdentity, PersistentNameIdentityOwnerPort as _,
};
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::CollectedMembers;
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameCommittedBatch};
use crate::lineage::naming_pages::builder::seal_pages;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::{
    BranchNamingRoot, CheckpointNamingState, GroupStateHeader, SchemeHeader, NAMING_KEY_BYTES,
    NAMING_PAGE_BYTES, NAMING_PAGE_ENTRIES,
};

#[derive(Debug, Default)]
struct MemSource {
    members: Mutex<BTreeMap<RelationalDurableMemberLocator, Vec<u8>>>,
    reads: AtomicUsize,
}

impl MemSource {
    fn from_sink(sink: CollectedMembers) -> Arc<Self> {
        Arc::new(Self {
            members: Mutex::new(sink.members.into_iter().collect()),
            reads: AtomicUsize::new(0),
        })
    }

    fn read_count(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}

impl RelationalDurableColdSource for MemSource {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let guard = self.members.lock().unwrap();
        let bytes = guard.get(&locator).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }

    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }

    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::MissingMember)
    }
}

fn synthetic_rows(count: usize, value_len: usize) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
    (0..count)
        .map(|i| {
            let mut key = vec![6u8];
            key.extend_from_slice(&(i as u64).to_be_bytes());
            let mut val = vec![(i & 0xff) as u8; value_len];
            val[0] = (i >> 8) as u8;
            (key, Some(val))
        })
        .collect()
}

#[test]
fn read_free_open_touched_faults_carry_and_eviction() {
    let mut sink = CollectedMembers::default();
    let rows = synthetic_rows(1200, 128);
    let pages = seal_pages(None, rows.clone(), &mut sink).unwrap();
    assert!(pages.len() >= 5, "multi-page naming run: {}", pages.len());
    let state = Arc::new(CheckpointNamingState {
        branches: vec![BranchNamingRoot {
            branch: BranchId("main".into()),
            naming: None,
            index: None,
            groups: None,
            pages: pages.clone(),
        }],
        group_log: Vec::new(),
        group_log_through: None,
    });
    assert!(state.validate().is_ok());

    let source = MemSource::from_sink(sink);
    let store = ColdNamingStore::open(Arc::clone(&state), source.clone());
    let budget = ResidencyBudget {
        head_working_set_bytes: u64::MAX / 4,
        historical_working_set_bytes: 96 * 1024,
        max_in_flight_fill_bytes: u64::MAX / 4,
        streaming_window_bytes: 64 * 1024,
    };
    let accountant = WorkingSetAccountant::new(budget);
    assert!(store.bind_working_set(Arc::clone(&accountant)));
    assert_eq!(source.read_count(), 0, "open must read zero pages");

    // Point lookup faults exactly one page.
    let mid_key = &rows[rows.len() / 2].0;
    let found = store.get(&pages, mid_key).unwrap();
    assert_eq!(found, rows[rows.len() / 2].1);
    assert_eq!(source.read_count(), 1, "point lookup faults one page");

    // Carry-forward with a single key edit touches only its page.
    let mut sink2 = CollectedMembers::default();
    let reads_before = source.read_count();
    let pages2 = seal_pages(
        Some((&store, &pages)),
        vec![(rows[0].0.clone(), Some(vec![0xaa; 128]))],
        &mut sink2,
    )
    .unwrap();
    assert_eq!(
        source.read_count() - reads_before,
        1,
        "editing one key reads only its touched page"
    );
    assert_eq!(
        &pages2[1..],
        &pages[1..],
        "untouched pages carry identical content addresses"
    );

    // Full scan under tight historical budget evicts and stays within budget.
    let mut seen = 0usize;
    store
        .scan(&pages, &[6u8], |_, _| {
            seen += 1;
            Ok(true)
        })
        .unwrap();
    assert_eq!(seen, rows.len());
    let pressure = accountant.pressure();
    assert!(pressure.historical_resident_bytes <= budget.historical_working_set_bytes);
    assert!(pressure.evictions_completed > 0);
}

#[test]
fn descriptor_validation_and_tampered_page_fail_closed() {
    let mut sink = CollectedMembers::default();
    let rows = synthetic_rows(300, 64);
    let pages = seal_pages(None, rows.clone(), &mut sink).unwrap();
    assert!(pages.len() >= 2);
    let source = MemSource::from_sink(sink);
    // Corrupt one byte of page 0 -> IntegrityMismatch.
    {
        let mut guard = source.members.lock().unwrap();
        let entry = guard.get_mut(&pages[0].member).unwrap();
        let last = entry.len() - 1;
        entry[last] ^= 0xff;
    }
    let state = Arc::new(CheckpointNamingState {
        branches: vec![BranchNamingRoot {
            branch: BranchId("main".into()),
            naming: None,
            index: None,
            groups: None,
            pages: pages.clone(),
        }],
        group_log: Vec::new(),
        group_log_through: None,
    });
    let store = ColdNamingStore::open(state, source);
    assert_eq!(
        store.get(&pages, &rows[0].0).unwrap_err(),
        Cause::IntegrityMismatch
    );

    // Unsorted / oversized / invalid descriptors refuse read-free.
    let mut bad_entries = pages[0].clone();
    bad_entries.entries = NAMING_PAGE_ENTRIES + 1;
    let bad_state = CheckpointNamingState {
        branches: vec![BranchNamingRoot {
            branch: BranchId("main".into()),
            naming: None,
            index: None,
            groups: Some(GroupStateHeader {
                earliest_commit: Some(1),
                latest_commit: Some(1),
                schemes: vec![SchemeHeader {
                    id: "s".into(),
                    rule: 1,
                }],
                unavailable: false,
            }),
            pages: vec![bad_entries],
        }],
        group_log: Vec::new(),
        group_log_through: None,
    };
    assert!(bad_state.validate().is_err());

    let mut bad_key = pages[0].clone();
    bad_key.first = vec![6u8; NAMING_KEY_BYTES + 1];
    assert!(super::validate_pages(&[bad_key]).is_err());

    let mut bad_len = pages[0].clone();
    bad_len.member.length = NAMING_PAGE_BYTES as u64 + 1;
    assert!(super::validate_pages(&[bad_len]).is_err());
}

#[test]
fn retention_floor_compaction_prunes_below_floor_and_preserves_ge_floor() {
    let mut index = PersistentNameBranchIndex::new(BranchId("main".into()), None);
    let group_id = AggregateIdentity::from_bytes([0x42; 32]);
    let group_key = PersistentNameKey::Group(group_id);
    let m1 = PersistentNameIdentity::from_minted_digest([0x11; 32]);
    let m2 = PersistentNameIdentity::from_minted_digest([0x22; 32]);
    let m3 = PersistentNameIdentity::from_minted_digest([0x33; 32]);

    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(1))
                .with_group_events([GroupEvent::Born { group: group_id }])
                .with_carriers(group_key, [m1, m2]),
        )
        .unwrap();
    for commit in 2..=20u64 {
        let carriers = if commit % 2 == 0 {
            vec![m1, m3]
        } else {
            vec![m1, m2]
        };
        index
            .apply(
                &PersistentNameCommittedBatch::new(CommitId(commit))
                    .with_carriers(group_key, carriers),
            )
            .unwrap();
    }

    let floor = CommitId(15);
    let oracle: Vec<_> = (15..=20)
        .map(|c| {
            (
                c,
                index.resolve(group_key, CommitId(c), &PersistentNameSelection::Exact),
            )
        })
        .collect();

    index.advance_retention_floor(floor).unwrap();
    assert_eq!(index.retention_floor(), Some(floor));
    for c in 1..15 {
        let err = index
            .resolve(group_key, CommitId(c), &PersistentNameSelection::Exact)
            .result
            .unwrap_err();
        assert!(matches!(
            err,
            PersistentNameResolutionDenial::BasisPruned { .. }
        ));
    }
    for (c, expected) in oracle {
        let actual = index.resolve(group_key, CommitId(c), &PersistentNameSelection::Exact);
        assert_eq!(actual.result, expected.result, "commit {c} equivalence");
    }
}
