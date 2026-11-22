//! Arch 21: every persisted sequence has one canonical byte order. Frames
//! whose events, holders, group events, carrier declarations, or carrier
//! sets are reordered or duplicated on the wire refuse `NonCanonical` before
//! any naming payload is readmitted.
//!
//! Frame layout: magic (4), version (1), payload length (4), then payload:
//! commit (8), retention-floor tag (1), and the counted sections in order.

use worth_foundational::facade::AggregateIdentity;

use crate::aggregate_identity::AggregateGroupLineageEvent;
use crate::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use crate::facade::identity::{EntityId, LineageId, PartitionId, RelationId, VersionId};
use crate::facade::lineage::{
    encode_commit_frame, encode_log_header, rebuild_name_index_from_durable_log, LineageEventKind,
    LineageEventRecord, LineageSubject, PersistentNameCommittedBatch,
    PersistentNameDurabilityError, PersistentNameIdentity, PersistentNameKey,
};

use super::name_index_fixtures::main_branch;

/// Payload start (9) plus commit (8) and an absent retention floor (1).
const SECTIONS: usize = 18;
const COUNT: usize = 4;
const HOLDER: usize = 8 + 4 + 8 + 4;
const GROUP_EVENT: usize = 1 + 32;
const CARRIER_DECLARATION: usize = 1 + 32 + COUNT + 2 * 32;

fn frame(batch: &PersistentNameCommittedBatch) -> Vec<u8> {
    encode_commit_frame(batch, None).expect("canonical batch encodes")
}

fn refusal(frame: &[u8]) -> PersistentNameDurabilityError {
    let mut log = encode_log_header(&main_branch()).unwrap();
    log.extend_from_slice(frame);
    rebuild_name_index_from_durable_log(&log).expect_err("non-canonical frame refuses")
}

fn non_canonical(field: &'static str) -> PersistentNameDurabilityError {
    PersistentNameDurabilityError::NonCanonical { field }
}

/// The frame with its two `width`-byte entries at `at` swapped, and with the
/// second overwritten by the first.
fn swapped_and_duplicated(frame: &[u8], at: usize, width: usize) -> [Vec<u8>; 2] {
    let mut swapped = frame.to_vec();
    swapped[at..at + 2 * width].rotate_left(width);
    let mut duplicated = frame.to_vec();
    duplicated.copy_within(at..at + width, at + width);
    [swapped, duplicated]
}

fn retire(event_id: u64) -> LineageEventRecord {
    let commit = RelationalCommitReceipt {
        commit_id: CommitId(1),
        version_id: VersionId(1),
        branch_id: main_branch(),
        parents: Vec::new(),
    };
    LineageEventRecord {
        event_id,
        commit,
        branch_id: BranchId("main".to_owned()),
        kind: LineageEventKind::Retire,
        sources: vec![LineageId(1)],
        targets: Vec::new(),
        subject: LineageSubject::Entity,
        name_birth: None,
        kept_name: None,
    }
}

fn group(byte: u8) -> AggregateIdentity {
    AggregateIdentity::from_bytes([byte; 32])
}

fn name(byte: u8) -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([byte; 32])
}

#[test]
fn reordered_or_duplicated_lineage_events_refuse_non_canonical() {
    let batch = |ids: &[u64]| {
        PersistentNameCommittedBatch::new(CommitId(1)).with_lineage_events(
            ids.iter().map(|id| retire(*id)),
            [],
            [],
        )
    };
    let (one, two) = (frame(&batch(&[1])), frame(&batch(&[1, 2])));
    let first = SECTIONS + COUNT;
    let second = first + (two.len() - one.len());
    let mut duplicated = two.clone();
    duplicated[second..second + 8].copy_from_slice(&1u64.to_be_bytes());
    let mut reordered = two.clone();
    reordered[first..first + 8].copy_from_slice(&3u64.to_be_bytes());
    for tampered in [duplicated, reordered] {
        assert_eq!(refusal(&tampered), non_canonical("lineage event order"));
    }
}

#[test]
fn reordered_or_duplicated_holders_refuse_non_canonical() {
    let entity = |slot| EntityId::new(PartitionId::main(), slot, 1);
    let relation = |slot| RelationId::new(PartitionId::main(), slot, 1);
    let entities = PersistentNameCommittedBatch::new(CommitId(1)).with_lineage_events(
        [],
        [(LineageId(1), entity(1)), (LineageId(2), entity(2))],
        [],
    );
    for tampered in swapped_and_duplicated(&frame(&entities), SECTIONS + 2 * COUNT, HOLDER) {
        assert_eq!(refusal(&tampered), non_canonical("entity holders"));
    }
    let relations = PersistentNameCommittedBatch::new(CommitId(1)).with_lineage_events(
        [],
        [],
        [(LineageId(1), relation(1)), (LineageId(2), relation(2))],
    );
    for tampered in swapped_and_duplicated(&frame(&relations), SECTIONS + 3 * COUNT, HOLDER) {
        assert_eq!(refusal(&tampered), non_canonical("relation holders"));
    }
}

#[test]
fn reordered_or_duplicated_group_events_refuse_non_canonical() {
    let batch = PersistentNameCommittedBatch::new(CommitId(1)).with_group_events([
        AggregateGroupLineageEvent::Born { group: group(1) },
        AggregateGroupLineageEvent::Born { group: group(2) },
    ]);
    let at = SECTIONS + 4 * COUNT;
    for tampered in swapped_and_duplicated(&frame(&batch), at, GROUP_EVENT) {
        assert_eq!(refusal(&tampered), non_canonical("group events"));
    }
}

#[test]
fn reordered_or_duplicated_carriers_refuse_non_canonical() {
    let batch = PersistentNameCommittedBatch::new(CommitId(1))
        .with_carriers(PersistentNameKey::Group(group(1)), [name(1), name(2)])
        .with_carriers(PersistentNameKey::Group(group(2)), [name(1), name(2)]);
    let encoded = frame(&batch);
    let declarations = SECTIONS + 5 * COUNT;
    for tampered in swapped_and_duplicated(&encoded, declarations, CARRIER_DECLARATION) {
        assert_eq!(refusal(&tampered), non_canonical("carrier declarations"));
    }
    let members = declarations + 1 + 32 + COUNT;
    for tampered in swapped_and_duplicated(&encoded, members, 32) {
        assert_eq!(refusal(&tampered), non_canonical("carriers"));
    }
}
