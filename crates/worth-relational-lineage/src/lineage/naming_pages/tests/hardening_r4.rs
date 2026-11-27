//! Round 4 hardening unit tests (`[P1-3]` and `[P1-4]`):
//! - `[P1-3]` Unnamed lineage retirement is recorded in `HolderEpoch::ended_at`,
//!   preserved across retention compaction, and excluded from `live_lineages`
//!   when truncating for a cold historical fork at or after retirement.
//! - Table-domain page isolation lives in `table_domains`.

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
use super::super::tables::{branch_key_digest, GROUP_LOG};
use super::super::{validate_pages, CheckpointNamingState};
use super::{CollectedMembers, MemSource};
use crate::lineage::index::{PersistentNameBranchIndex, PersistentNameCommittedBatch};
use crate::runtime::state::subsystems::lineage::naming_basis::AuthoritativeBranchNamingState;

pub(crate) fn unnamed_event(
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
    let (e1, l1) = (EntityId::new(PartitionId::main(), 101, 1), LineageId(1));
    let (e2, l2) = (EntityId::new(PartitionId::main(), 202, 1), LineageId(2));
    let mut index = PersistentNameBranchIndex::new(branch.clone(), None);
    let ev = |id, c, k, s, t| unnamed_event(id, &branch, CommitId(c), k, s, t);
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(1)).with_lineage_events(
                [ev(1, 1, LineageEventKind::Create, vec![], vec![l1])],
                [(l1, e1)],
                [],
            ),
        )
        .unwrap();
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(2)).with_lineage_events(
                [ev(2, 2, LineageEventKind::Retire, vec![l1], vec![])],
                [],
                [],
            ),
        )
        .unwrap();
    index
        .apply(
            &PersistentNameCommittedBatch::new(CommitId(3)).with_lineage_events(
                [ev(3, 3, LineageEventKind::Create, vec![], vec![l2])],
                [(l2, e2)],
                [],
            ),
        )
        .unwrap();
    let basis_at = |idx: &PersistentNameBranchIndex, name: &str, c: u64| {
        let child = idx.truncate_at(BranchId(name.into()), CommitId(c)).unwrap();
        assert_eq!(child.head(), Some(CommitId(c)));
        AuthoritativeBranchNamingState::from_truncated_index(&child, CommitId(c))
            .unwrap()
            .snapshot()
            .unwrap()
    };
    let b1 = basis_at(&index, "c1", 1);
    assert_eq!(
        b1.live_lineages.get(&PersistentNameHolder::Entity(e1)),
        Some(&l1)
    );
    assert!(!b1
        .live_lineages
        .contains_key(&PersistentNameHolder::Entity(e2)));
    let b2 = basis_at(&index, "c2", 2);
    assert!(!b2
        .live_lineages
        .contains_key(&PersistentNameHolder::Entity(e1)));
    index.advance_retention_floor(CommitId(2)).unwrap();
    let b2_compact = basis_at(&index, "c2-compact", 2);
    assert!(!b2_compact
        .live_lineages
        .contains_key(&PersistentNameHolder::Entity(e1)));
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
