use super::*;
use crate::cold_source::{test_store, StoreColdSource};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::RelationalDurableColdSource;
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublished,
    PhysicalMutationDeadline,
};

fn publish(source: &StoreColdSource, generation: u64, bytes: &[u8]) -> AnchoredRecordPublished {
    source
        .serving()
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                generation,
                b"original selected payload".to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(vec![AnchoredRecordMember::New {
                key: Sha256::digest(bytes).into(),
                bytes: bytes.to_vec(),
            }]),
        )
        .unwrap()
}
fn key(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}

#[test]
fn selected_replacement_keeps_original_candidate_through_busy_and_releases_old_local_read() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving);
    let cancellation = CancellationSource::new();
    let serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let old_bytes = vec![19; 130_001];
    let new_bytes = vec![81; 140_003];
    let first = publish(&source, 0, &old_bytes);
    let read = read_expected(&source, first.version(), &request).unwrap();
    source
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    drop(read);
    let old = source
        .checkpoint_selected_read_protection(&request)
        .unwrap();
    let second = publish(&source, 1, &new_bytes);
    let held = source.members.read().unwrap();
    let mut task = SelectedReplacement::default();
    assert!(matches!(
        task.advance(&source, second.version(), &[key(&new_bytes)], &request),
        Err(Denial::Busy)
    ));
    let candidate = task.displaced.as_ref().unwrap();
    assert_eq!(
        candidate.read.as_ref().unwrap().membership().version(),
        second.version()
    );
    assert_eq!(candidate.entries.len(), 1);
    assert_eq!(held.version, Some(first.version()));
    drop(held);
    task.advance(&source, second.version(), &[key(&new_bytes)], &request)
        .unwrap();
    assert!(task.complete);
    assert!(task._retired.is_some());
    assert_eq!(
        source.members.read().unwrap().version,
        Some(second.version())
    );
    assert_eq!(
        &*source
            .read_member_owned(&request, old.as_ref(), key(&old_bytes), 1 << 20)
            .unwrap(),
        &old_bytes
    );
    let now = source
        .checkpoint_selected_read_protection(&request)
        .unwrap();
    assert_eq!(
        &*source
            .read_member_owned(&request, now.as_ref(), key(&new_bytes), 1 << 20)
            .unwrap(),
        &new_bytes
    );
    assert!(source
        .read_member_owned(&request, now.as_ref(), key(&old_bytes), 1 << 20)
        .is_err());
    assert!(source.select_members(first.membership()).is_err());
    cancellation.cancel();
    // A completed local replacement performs no new request polling.
    task.advance(&source, second.version(), &[], &request)
        .unwrap();
}

#[test]
fn selected_replacement_stop_after_eof_preserves_same_read_and_placement() {
    let directory = tempfile::tempdir().unwrap();
    let source = StoreColdSource::new(Arc::new(test_store::initialize(directory.path())));
    let cancellation = CancellationSource::new();
    let serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let first = publish(&source, 0, b"old member");
    let read = read_expected(&source, first.version(), &request).unwrap();
    source
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    drop(read);
    let second = publish(&source, 1, b"new member");
    let mut task = SelectedReplacement::default();
    let held = source.members.read().unwrap();
    assert!(matches!(
        task.advance(&source, second.version(), &[], &request),
        Err(Denial::Busy)
    ));
    drop(held);
    cancellation.cancel();
    assert!(matches!(
        task.advance(&source, second.version(), &[], &request),
        Err(Denial::Execution(_))
    ));
    assert_eq!(
        source.members.read().unwrap().version,
        Some(first.version())
    );
    assert_eq!(
        task.displaced
            .as_ref()
            .unwrap()
            .read
            .as_ref()
            .unwrap()
            .membership()
            .version(),
        second.version()
    );
    assert!(!task.installed);
    // Original verified data remains owned; abandonment is explicitly unbudgeted.
    drop(task);
}

#[test]
fn selected_read_and_replacement_refuse_foreign_source_with_same_serving() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving.clone());
    let foreign = StoreColdSource::new(serving);
    let cancellation = CancellationSource::new();
    let serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let request = ExecutionRequest::serial(&serial);
    let first = publish(&source, 0, b"original");
    let read = read_expected(&source, first.version(), &request).unwrap();
    source
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    foreign
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    drop(read);
    let original = source
        .checkpoint_selected_read_protection(&request)
        .unwrap();
    let before = foreign.counters();
    assert!(matches!(
        foreign.read_member_owned(&request, original.as_ref(), key(b"original"), 1 << 20),
        Err(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal::ForeignSource)
    ));
    assert_eq!(foreign.counters().member_reads, before.member_reads);
    let second = publish(&source, 1, b"successor");
    let mut task = SelectedReplacement::default();
    let held = source.members.read().unwrap();
    assert!(matches!(
        task.advance(&source, second.version(), &[], &request),
        Err(Denial::Busy)
    ));
    drop(held);
    assert!(matches!(
        task.advance(&foreign, second.version(), &[], &request),
        Err(Denial::DifferentSelection)
    ));
    assert_eq!(
        foreign.members.read().unwrap().version,
        Some(first.version())
    );
    task.advance(&source, second.version(), &[], &request)
        .unwrap();
}
