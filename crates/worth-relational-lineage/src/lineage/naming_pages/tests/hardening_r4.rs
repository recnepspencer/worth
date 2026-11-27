//! Round 4 hardening unit tests (`[P1-3]` and `[P1-4]`):
//! - `[P1-3]` Unnamed lineage retirement is recorded in `HolderEpoch::ended_at`,
//!   preserved across retention compaction, and excluded from `live_lineages`
//!   when truncating for a cold historical fork at or after retirement.
//! - `[P1-4]` `seal_pages` isolates every physical page to one table domain
//!   with zero unnecessary reads across domains, and `validate_pages` refuses
//!   multi-domain or unknown-tag descriptors read-free.

use std::sync::Arc;

use worth_relational_contracts::internal_port::history::data::commit_receipt::RelationalCommitReceipt;
use worth_relational_contracts::internal_port::history::data::coordinates::{BranchId, CommitId};
use worth_relational_contracts::internal_port::identity::data::primitives::{
    LineageId, PartitionId, VersionId,
};
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;
use worth_relational_contracts::internal_port::lineage::data::events::{
    assemble_lineage_event_record, LineageEventKind, LineageEventRecord, LineageEventRecordParts,
    LineageSubject,
};
use worth_relational_contracts::internal_port::lineage::data::naming::PersistentNameHolder;

use super::super::builder::seal_pages;
use super::super::codec::NamingCodec;
use super::super::store::ColdNamingStore;
use super::super::tables::{
    branch_key_digest, table_domain, GROUP_LIVE, GROUP_LOG, INDEX_NAME, LINEAGE_HOLDER,
    MAX_INLINE_VALUE_BYTES, VALUE_CHUNK,
};
use super::super::{validate_pages, BranchNamingRoot, CheckpointNamingState};
use super::{CollectedMembers, MemSource};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameCommittedBatch};
use crate::runtime::state::subsystems::lineage::naming_basis::AuthoritativeBranchNamingState;

fn unnamed_event(
    event_id: u64,
    branch: &BranchId,
    commit: CommitId,
    kind: LineageEventKind,
    sources: Vec<LineageId>,
    targets: Vec<LineageId>,
) -> LineageEventRecord {
    assemble_lineage_event_record(LineageEventRecordParts {
        event_id,
        commit: RelationalCommitReceipt {
            commit_id: commit,
            version_id: VersionId(commit.0),
            branch_id: branch.clone(),
            parents: Vec::new(),
        },
        branch_id: branch.clone(),
        kind,
        sources,
        targets,
        subject: LineageSubject::Entity,
        name_birth: None,
        kept_name: None,
        relation_id: None,
        entity_id: None,
    })
}

#[test]
fn retired_unnamed_lineage_not_resurrected_in_truncated_naming_basis() {
    let branch = BranchId("main".into());
    let entity = EntityId::new(PartitionId::main(), 101, 1);
    let lineage = LineageId(1);
    let entity2 = EntityId::new(PartitionId::main(), 202, 1);
    let lineage2 = LineageId(2);
    let mut index = PersistentNameBranchIndex::new(branch.clone(), None);

    // C1: create unnamed entity1 (lineage 1).
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(1)).with_lineage_events(
                [unnamed_event(
                    1,
                    &branch,
                    CommitId(1),
                    LineageEventKind::Create,
                    vec![],
                    vec![lineage],
                )],
                [(lineage, entity)],
                [],
            ),
        )
        .unwrap();

    // C2: retire unnamed entity1 (lineage 1).
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(2)).with_lineage_events(
                [unnamed_event(
                    2,
                    &branch,
                    CommitId(2),
                    LineageEventKind::Retire,
                    vec![lineage],
                    vec![],
                )],
                [],
                [],
            ),
        )
        .unwrap();

    // C3: create another unnamed entity2 (lineage 2).
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(3)).with_lineage_events(
                [unnamed_event(
                    3,
                    &branch,
                    CommitId(3),
                    LineageEventKind::Create,
                    vec![],
                    vec![lineage2],
                )],
                [(lineage2, entity2)],
                [],
            ),
        )
        .unwrap();

    // Truncate at C1: entity1 MUST be live; entity2 not yet born.
    let child_c1 = index
        .truncate_at(BranchId("c1".into()), CommitId(1))
        .unwrap();
    let basis_c1 = AuthoritativeBranchNamingState::from_truncated_index(&child_c1, CommitId(1))
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(
        basis_c1
            .live_lineages
            .get(&PersistentNameHolder::Entity(entity)),
        Some(&lineage),
        "unnamed entity1 must be live at C1 before its C2 retirement"
    );
    assert!(!basis_c1
        .live_lineages
        .contains_key(&PersistentNameHolder::Entity(entity2)));

    // Truncate at C2: entity1 was retired at C2 so MUST NOT be resurrected in live_lineages!
    let child_c2 = index
        .truncate_at(BranchId("c2".into()), CommitId(2))
        .unwrap();
    assert_eq!(child_c2.head(), Some(CommitId(2)));
    let basis_c2 = AuthoritativeBranchNamingState::from_truncated_index(&child_c2, CommitId(2))
        .unwrap()
        .snapshot()
        .unwrap();
    assert!(
        !basis_c2
            .live_lineages
            .contains_key(&PersistentNameHolder::Entity(entity)),
        "retired unnamed entity1 must not be resurrected at C2"
    );

    // Retention compaction at C2 preserves ended_at = Some(C2) on the kept epoch.
    index.advance_retention_floor(CommitId(2)).unwrap();
    let child_c2_after_compact = index
        .truncate_at(BranchId("c2-compact".into()), CommitId(2))
        .unwrap();
    let basis_c2_after_compact =
        AuthoritativeBranchNamingState::from_truncated_index(&child_c2_after_compact, CommitId(2))
            .unwrap()
            .snapshot()
            .unwrap();
    assert!(
        !basis_c2_after_compact
            .live_lineages
            .contains_key(&PersistentNameHolder::Entity(entity)),
        "retention compaction must preserve unnamed retirement ended_at"
    );
}

#[test]
fn naming_pages_isolate_domains_without_cross_domain_reads_and_refuse_mixed_descriptors() {
    let k_naming = vec![LINEAGE_HOLDER, 1, 2, 3];
    let k_index = vec![INDEX_NAME, 1, 2, 3];
    let k_group = vec![GROUP_LIVE, 1, 2, 3];

    // Initial seal with domains 1 and 3 (plus a chunked value in domain 3 -> domain 5).
    let mut sink1 = CollectedMembers::default();
    let big = vec![0x77u8; MAX_INLINE_VALUE_BYTES + 128];
    let pages1 = seal_pages(
        None,
        vec![
            (k_naming.clone(), Some(vec![1u8; 8])),
            (k_group.clone(), Some(big.clone())),
        ],
        &mut sink1,
    )
    .unwrap();
    assert_eq!(
        pages1.len(),
        3,
        "domains 1, 3, and 5 must occupy separate pages"
    );
    for page in &pages1 {
        assert_eq!(
            table_domain(page.first[0]),
            table_domain(page.last[0]),
            "every sealed page must stay within a single table domain"
        );
    }
    assert_eq!(table_domain(pages1[0].first[0]), Some(1));
    assert_eq!(table_domain(pages1[1].first[0]), Some(3));
    assert_eq!(pages1[2].first[0], VALUE_CHUNK);
    assert!(validate_pages(&pages1).is_ok());

    // Incremental seal adding a domain-2 row between domain 1 and domain 3
    // must read ZERO prior pages and carry domains 1, 3, and 5 untouched.
    let source = MemSource::from_sink(sink1);
    let store = ColdNamingStore::open(
        Arc::new(CheckpointNamingState {
            branches: vec![BranchNamingRoot {
                branch: BranchId("main".into()),
                naming: None,
                index: None,
                groups: None,
                pages: pages1.clone(),
            }],
            group_log: Vec::new(),
            group_log_through: None,
        }),
        source.clone(),
    );
    let mut sink2 = CollectedMembers::default();
    let pages2 = seal_pages(
        Some((&store, &pages1)),
        vec![(k_index.clone(), Some(vec![2u8; 8]))],
        &mut sink2,
    )
    .unwrap();
    assert_eq!(
        source.read_count(),
        0,
        "inserting a row into a new table domain must read zero untouched prior pages"
    );
    assert_eq!(pages2.len(), 4);
    assert_eq!(pages2[0], pages1[0]);
    assert_eq!(table_domain(pages2[1].first[0]), Some(2));
    assert_eq!(pages2[2], pages1[1]);
    assert_eq!(pages2[3], pages1[2]);
    assert!(validate_pages(&pages2).is_ok());

    // Synthetic mixed-domain descriptor refuses read-free.
    let mut mixed = pages1[0].clone();
    mixed.last = k_index;
    mixed.entries = 2;
    assert!(validate_pages(&[mixed]).is_err());

    // Unknown table tag descriptor refuses read-free.
    let mut unknown = pages1[0].clone();
    unknown.first = vec![0u8, 1];
    unknown.last = vec![0u8, 1];
    unknown.entries = 1;
    assert!(validate_pages(&[unknown]).is_err());
}

#[test]
fn seal_pages_appending_different_branch_group_log_carries_prior_page_without_reads() {
    let key = |ord: u64, c: u64, b: &str| {
        let mut k = vec![GROUP_LOG];
        let digest = branch_key_digest(&BranchId(b.into()));
        k.extend_from_slice(&((ord, CommitId(c)), digest).to_bytes());
        k
    };
    let mut sink1 = CollectedMembers::default();
    let pages1 = seal_pages(None, vec![(key(0, 1, "a"), Some(vec![1; 8]))], &mut sink1).unwrap();
    let source = MemSource::from_sink(sink1);
    source.members.lock().unwrap().remove(&pages1[0].member);
    let state = CheckpointNamingState {
        branches: Vec::new(),
        group_log: pages1.clone(),
        group_log_through: Some(1),
    };
    let store = ColdNamingStore::open(Arc::new(state), source.clone());
    let mut sink2 = CollectedMembers::default();
    let pages2 = seal_pages(
        Some((&store, &pages1)),
        vec![(key(1, 2, "b"), Some(vec![2; 8]))],
        &mut sink2,
    )
    .unwrap();
    assert_eq!(source.read_count(), 0);
    assert_eq!(pages2.len(), 2);
    assert_eq!(pages2[0], pages1[0]);
    assert!(validate_pages(&pages2).is_ok());
}
