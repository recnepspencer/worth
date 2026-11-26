//! Paged commit index: read-free open, touched-page faults, bounded
//! residency, prefix carry-forward and typed refusal of damaged pages.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::builder::CommitIndexBuilder;
use super::reader::ColdCommitIndex;
use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};
use crate::history::data::{BranchId, RelationalAssociationDiscovery, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use worth_relational_contracts::internal_port::config::data::residency_budget::ResidencyBudget;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::storage::residency::chunked::cold_member_port::{
    ColdMemberSource, CollectedMembers,
};
use worth_relational_storage::internal_port::storage::residency::chunked::member_frame::{
    encode_member, ChunkMemberKind,
};
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

#[derive(Debug, Default)]
struct Members {
    bytes: Mutex<BTreeMap<RelationalDurableMemberLocator, Vec<u8>>>,
    reads: AtomicUsize,
}

impl RelationalDurableColdSource for Members {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.bytes
            .lock()
            .unwrap()
            .get(&locator)
            .cloned()
            .ok_or(Cause::MissingMember)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

/// Commit ids interleave within each page, as concurrent preparation allows.
fn commit_of(position: u64) -> u64 {
    if position % 2 == 1 {
        position + 11
    } else {
        position + 9
    }
}

fn entry(position: u64) -> RelationalCommitIndexEntry {
    let commit = commit_of(position);
    RelationalCommitIndexEntry {
        association_discovery: RelationalAssociationDiscovery::Unassociated,
        commit: RelationalCommitReceipt {
            commit_id: CommitId(commit),
            version_id: VersionId(position),
            branch_id: BranchId(format!("branch-{}", position % 3)),
            parents: (position > 1)
                .then(|| CommitId(commit_of(position - 1)))
                .into_iter()
                .collect(),
        },
        position: PatchStreamPosition(position),
        root: crate::branch::target::RelationalBranchRootDescriptor::new([3; 32], [4; 32]),
        payload_digest: [position as u8; 32],
        schema_version: crate::schema::data::SchemaVersionId(1),
    }
}

fn build(
    members: &Members,
    prior: Option<&CheckpointCommitIndex>,
    count: u64,
    roots: &[u64],
) -> CheckpointCommitIndex {
    let mut sink = CollectedMembers::default();
    let carried = prior.map_or(&[][..], |prior| {
        &prior.pages[..prior.pages.len().saturating_sub(1)]
    });
    let roots: BTreeSet<_> = roots.iter().map(|&p| entry(p).commit.commit_id).collect();
    let mut builder = CommitIndexBuilder::resume(&mut sink, carried, roots).unwrap();
    let start = carried.last().map_or(1, |page| page.end().unwrap());
    for position in start..=count {
        builder.push(entry(position)).unwrap();
    }
    let carried_heads = builder
        .unresolved_heads()
        .map(|commit| prior.unwrap().head(commit).unwrap().clone())
        .collect();
    let index = builder
        .finish(prior.and_then(|p| p.tail.clone()), carried_heads)
        .unwrap();
    members.bytes.lock().unwrap().extend(sink.members);
    index
}

fn tight() -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: 1 << 20,
        historical_working_set_bytes: 96 * 1024,
        max_in_flight_fill_bytes: 1 << 20,
        streaming_window_bytes: 1 << 16,
    })
}

#[test]
fn long_history_opens_without_reads_and_faults_only_touched_pages() {
    for scale in [1_u64, 10] {
        let members = Arc::new(Members::default());
        let count = 300 * scale;
        let index = build(&members, None, count, &[7, count]);
        index.validate().unwrap();
        let reader = ColdCommitIndex::open(Arc::new(index.clone()), members.clone());
        reader.bind_working_set(tight());
        assert_eq!(
            members.reads.load(Ordering::SeqCst),
            0,
            "open reads nothing"
        );
        // Heads and the tail answer inline for position, commit, and version.
        assert_eq!(
            reader.by_commit(entry(7).commit.commit_id).unwrap(),
            Some(entry(7))
        );
        assert_eq!(
            reader.by_version(entry(7).commit.version_id).unwrap(),
            Some(entry(7))
        );
        assert_eq!(
            reader.at_position(PatchStreamPosition(count)).unwrap(),
            Some(entry(count))
        );
        assert_eq!(
            reader.by_version(entry(count).commit.version_id).unwrap(),
            Some(entry(count))
        );
        assert_eq!(reader.observation().faults, 0);
        // One lookup by position, commit or version faults exactly its page.
        let probe = count / 2;
        assert_eq!(
            reader.at_position(PatchStreamPosition(probe)).unwrap(),
            Some(entry(probe))
        );
        assert_eq!(
            reader.by_commit(entry(probe + 1).commit.commit_id).unwrap(),
            Some(entry(probe + 1))
        );
        assert_eq!(
            reader.by_version(VersionId(probe)).unwrap(),
            Some(entry(probe))
        );
        assert!(reader.observation().faults <= 2);
        assert_eq!(reader.by_commit(CommitId(u64::MAX)).unwrap(), None);
        // A full scan stays within the historical pool by evicting.
        let mut seen = 0;
        reader
            .try_for_each(|row| {
                seen += 1;
                assert_eq!(row, &entry(row.position.0));
                Ok::<_, Cause>(())
            })
            .unwrap();
        assert_eq!(seen, count);
        let observation = reader.observation();
        // The same tight pool bounds residency at either history length.
        assert!(observation.resident_pages <= 2, "{observation:?}");
        assert!(observation.evictions > 0);
        assert_eq!(
            reader.after(probe, 3).unwrap(),
            (probe + 1..probe + 4).map(entry).collect::<Vec<_>>()
        );
    }
}

#[test]
fn appended_checkpoints_carry_every_full_prefix_page() {
    let members = Members::default();
    let first = build(&members, None, 300, &[300]);
    let second = build(&members, Some(&first), 420, &[300, 420]);
    let third = build(&members, Some(&second), 420, &[420]);
    for index in [&first, &second, &third] {
        index.validate().unwrap();
    }
    let full = first.pages.len() - 1;
    assert_eq!(&second.pages[..full], &first.pages[..full]);
    // The rewritten tail page supersedes the prior one.
    assert_ne!(second.pages[full], first.pages[full]);
    // An unchanged history reproduces the same members.
    assert_eq!(third.pages, second.pages);
    assert_eq!(second.head(entry(300).commit.commit_id), Some(&entry(300)));
}

#[test]
fn damaged_pages_and_descriptors_refuse_without_panicking() {
    let members = Arc::new(Members::default());
    let index = build(&members, None, 300, &[300]);
    // A descriptor naming a member outside the admitted roster is refused
    // by confinement before any read, even though the source holds it.
    let foreign = encode_member(
        ChunkMemberKind::CommitIndexPage,
        &page::CommitIndexPage {
            first: 1,
            rows: (1..=127).map(entry).collect(),
        },
        COMMIT_PAGE_BYTES,
    )
    .unwrap();
    members
        .bytes
        .lock()
        .unwrap()
        .insert(foreign.locator, foreign.bytes);
    let mut unlisted = index.clone();
    unlisted.pages[0].member = foreign.locator;
    let roster = index.page_members().collect();
    let source = ColdMemberSource::new(members.clone(), COMMIT_PAGE_BYTES as u64);
    let confined = ColdCommitIndex::confined(Arc::new(unlisted), source, roster);
    let reads = members.reads.load(Ordering::SeqCst);
    assert_eq!(confined.page(0).unwrap_err(), Cause::Damaged);
    assert_eq!(
        confined.page(index.pages.len()).unwrap_err(),
        Cause::Damaged
    );
    assert_eq!(members.reads.load(Ordering::SeqCst), reads);
    // A page whose rows disagree with the descriptor is an integrity refusal
    // and must not remain cached in the working-set pool.
    let mut shifted = index.clone();
    shifted.pages[0].commits[1] += 1;
    let shifted = ColdCommitIndex::open(Arc::new(shifted), members.clone());
    assert_eq!(shifted.page(0).unwrap_err(), Cause::IntegrityMismatch);
    assert_eq!(shifted.observation().resident_pages, 0);
    // Tampered head at tail.position and duplicate/zero version/commit ids refuse.
    let mut tampered_head = index.clone();
    tampered_head.heads[0].payload_digest[0] ^= 0xff;
    assert!(tampered_head.validate().is_err());
    let mut zero_version = index.clone();
    zero_version.pages[0].versions[0] = 0;
    assert!(zero_version.validate().is_err());
    let mut dup_rows: Vec<_> = (1..=4).map(entry).collect();
    dup_rows[2].commit.commit_id = dup_rows[0].commit.commit_id;
    let dup_page = encode_member(
        ChunkMemberKind::CommitIndexPage,
        &page::CommitIndexPage {
            first: 1,
            rows: dup_rows,
        },
        COMMIT_PAGE_BYTES,
    )
    .unwrap();
    let dup_index = CheckpointCommitIndex {
        count: 4,
        tail: Some(entry(4)),
        heads: Vec::new(),
        pages: vec![CommitIndexPageRef {
            first: 1,
            entries: 4,
            commits: [11, 14],
            versions: [1, 4],
            member: dup_page.locator,
        }],
    };
    members
        .bytes
        .lock()
        .unwrap()
        .insert(dup_page.locator, dup_page.bytes);
    assert_eq!(
        ColdCommitIndex::open(Arc::new(dup_index), members.clone())
            .page(0)
            .unwrap_err(),
        Cause::Damaged
    );
    // Unsorted rows and an unknown wire field are damaged frames.
    let mut rows: Vec<_> = (1..=4).map(entry).collect();
    rows.swap(1, 2);
    let unsorted = encode_member(
        ChunkMemberKind::CommitIndexPage,
        &page::CommitIndexPage { first: 1, rows },
        COMMIT_PAGE_BYTES,
    )
    .unwrap();
    #[derive(serde::Serialize)]
    struct Extended {
        first: u64,
        rows: Vec<RelationalCommitIndexEntry>,
        extra: u64,
    }
    let extended = encode_member(
        ChunkMemberKind::CommitIndexPage,
        &Extended {
            first: 1,
            rows: (1..=4).map(entry).collect(),
            extra: 9,
        },
        COMMIT_PAGE_BYTES,
    )
    .unwrap();
    for member in [unsorted, extended] {
        let tampered = CheckpointCommitIndex {
            count: 4,
            tail: Some(entry(4)),
            heads: Vec::new(),
            pages: vec![CommitIndexPageRef {
                first: 1,
                entries: 4,
                commits: [11, 14],
                versions: [1, 4],
                member: member.locator,
            }],
        };
        members
            .bytes
            .lock()
            .unwrap()
            .insert(member.locator, member.bytes);
        let reader = ColdCommitIndex::open(Arc::new(tampered), members.clone());
        assert_eq!(reader.page(0).unwrap_err(), Cause::Damaged);
    }
    // Read-free descriptor validation.
    let mut gap = index.clone();
    gap.pages[1].first += 1;
    assert!(gap.validate().is_err());
    let mut short = index.clone();
    short.count += 1;
    assert!(short.validate().is_err());
    let mut repeated = index.clone();
    repeated.pages[1].member = repeated.pages[0].member;
    assert!(repeated.validate().is_err());
    let mut outside = index.clone();
    outside.tail.as_mut().unwrap().commit.commit_id = CommitId(1);
    assert!(outside.validate().is_err());
    let reader = ColdCommitIndex::open(Arc::new(index), members);
    assert_eq!(reader.at_position(PatchStreamPosition(301)).unwrap(), None);
    // Single-pass uncached page reads verify and decode without pinning cache slots.
    let uncached = reader.read_page_uncached(0).unwrap();
    assert_eq!(uncached.page.rows.len(), 128);
    assert_eq!(reader.observation().resident_pages, 0);
    assert_eq!(reader.observation().faults, 0);
}
