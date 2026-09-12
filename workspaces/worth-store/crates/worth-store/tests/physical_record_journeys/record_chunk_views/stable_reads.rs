use std::num::NonZeroU32;

use worth_store::physical_runtime::{
    PhysicalReadProtectionDenial, PhysicalReadProtectionDisposition, PhysicalReadProtectionPolicy,
    PhysicalRecordId, PhysicalRecordScanSession, RecordByteLimit, RecordCountLimit,
    RecordReadDenial, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
    RecordStreamFailureKind,
};

use super::fixture;
use super::managed_append::append;

#[test]
fn old_membership_and_cold_extent_survive_managed_publication() {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("stable-root"));
    let expected = fixture::payload(3 * fixture::CHUNK_PAYLOAD_BYTES + 19);
    let first = append(&serving, placement, 201, b"first");
    let second = append(&serving, placement, 202, b"second");
    let extent = append(&serving, placement, 203, &expected);
    let mut old_membership = vec![first, second, extent];
    old_membership.sort();
    let old = serving.records().unwrap();
    let protected = old.protected_root();
    let observer = serving.read_protection_observer();
    let mut scan = serving.records().unwrap().scan(scan_request()).unwrap();
    let mut ids = Vec::new();
    let mut scratch = vec![0; 65_536];
    match scan.read_next_into(&mut scratch).unwrap() {
        RecordScanOutcome::Batch(batch) => {
            ids.extend(batch.records().iter().map(|record| record.record_id()))
        }
        _ => panic!("the old scan must start with the first record"),
    }
    assert_eq!(ids, old_membership[..1]);
    let mut session = old.open(extent, limits(expected.len())).unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        &expected[..fixture::CHUNK_PAYLOAD_BYTES]
    );
    let appended = append(&serving, placement, 204, b"new-root-only");
    assert!(
        matches!(old.open(appended, limits(100)), Err(error) if error.denial() == RecordReadDenial::RecordNotFound)
    );
    assert_eq!(observer.acquisitions_for_root(protected), 2);
    ids.extend(collect_ids(&mut scan));
    assert_eq!(ids, old_membership);
    drop(scan);
    drop(old);
    assert_eq!(observer.acquisitions_for_root(protected), 1);
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let before = session.observation().physical_work_count();
    let mut offset = fixture::CHUNK_PAYLOAD_BYTES;
    {
        let second_chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(
            second_chunk.bytes(),
            &expected[offset..offset + second_chunk.bytes().len()]
        );
        offset += second_chunk.bytes().len();
    }
    assert!(
        session.observation().physical_work_count() > before,
        "the second chunk must execute a cold physical read"
    );
    while let Some(chunk) = session.next_chunk().unwrap() {
        assert_eq!(
            chunk.bytes(),
            &expected[offset..offset + chunk.bytes().len()]
        );
        offset += chunk.bytes().len();
    }
    assert_eq!(offset, expected.len());
    drop(session);
    assert_eq!(observer.acquisitions_for_root(protected), 0);
    let current = serving.records().unwrap();
    assert_ne!(current.protected_root().root(), protected.root());
    let mut current_scan = current.scan(scan_request()).unwrap();
    let mut new_membership = vec![first, second, extent, appended];
    new_membership.sort();
    assert_eq!(collect_ids(&mut current_scan), new_membership);
    drop(current_scan);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    fixture::assert_clean_close(serving);
}

#[test]
fn detached_session_owns_the_only_slot_until_final_release() {
    let parent = tempfile::tempdir().unwrap();
    let one = NonZeroU32::new(1).unwrap();
    let (serving, placement) = fixture::initialize_with_protection(
        &parent.path().join("one-slot"),
        PhysicalReadProtectionPolicy::new(one, one),
    );
    let expected = fixture::payload(2 * fixture::CHUNK_PAYLOAD_BYTES + 7);
    let record = append(&serving, placement, 205, &expected);
    let observer = serving.read_protection_observer();
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    let reader = serving.records().unwrap();
    let root = reader.protected_root();
    let mut session = reader.open(record, limits(expected.len())).unwrap();
    drop(reader);
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(PhysicalReadProtectionDenial::ProtectionLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    assert_eq!(observer.acquisitions_for_root(root), 1);
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        &expected[..fixture::CHUNK_PAYLOAD_BYTES]
    );
    drop(session);
    assert_eq!(observer.acquisitions_for_root(root), 0);
    assert_eq!(observer.snapshot().releases(), 1);
    let next = serving.records().unwrap();
    assert_eq!(observer.acquisitions_for_root(next.protected_root()), 1);
    drop(next);
    assert_eq!(observer.snapshot().releases(), 2);
    fixture::assert_clean_close(serving);
}

#[test]
fn borrowed_chunk_survives_close_and_no_new_view_is_issued() {
    borrowed_shutdown(false);
}

#[test]
fn distinct_root_pressure_preserves_both_acquisitions_of_the_old_root() {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize_with_protection(
        &parent.path().join("root-budget"),
        PhysicalReadProtectionPolicy::new(NonZeroU32::new(3).unwrap(), NonZeroU32::new(1).unwrap()),
    );
    append(&serving, placement, 207, b"old");
    let first = serving.records().unwrap();
    let second = serving.records().unwrap();
    let root = first.protected_root();
    let observer = serving.read_protection_observer();
    assert_eq!(observer.acquisitions_for_root(root), 2);
    append(&serving, placement, 208, b"new");
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(PhysicalReadProtectionDenial::RetainedRootLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    drop(first);
    assert_eq!(observer.acquisitions_for_root(root), 1);
    assert!(matches!(
        serving.records(),
        Err(PhysicalReadProtectionDenial::RetainedRootLimit)
    ));
    drop(second);
    assert_eq!(observer.acquisitions_for_root(root), 0);
    let current = serving.records().unwrap();
    assert_ne!(current.protected_root().root(), root.root());
    drop(current);
    fixture::assert_clean_close(serving);
}

#[test]
fn implicit_runtime_drop_revokes_sessions_without_releasing_borrowed_memory() {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("implicit-drop"));
    let record = append(&serving, placement, 209, b"retained bytes");
    let observer = serving.read_protection_observer();
    let mut session = serving
        .records()
        .unwrap()
        .open(record, limits(100))
        .unwrap();
    let chunk = session.next_chunk().unwrap().unwrap();
    drop(serving);
    assert!(observer.snapshot().revoked());
    assert_eq!(chunk.bytes(), b"retained bytes");
    drop(chunk);
    assert!(
        matches!(session.next_chunk(), Err(error) if error.kind() == RecordStreamFailureKind::RuntimeReleased)
    );
    drop(session);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    assert_eq!(observer.snapshot().releases(), 1);
}

#[test]
fn borrowed_chunk_survives_abort_and_no_new_view_is_issued() {
    borrowed_shutdown(true);
}

#[test]
fn reopen_issues_fresh_protection_while_old_incarnation_remains_revoked() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("fresh-incarnation");
    let one = NonZeroU32::new(1).unwrap();
    let policy = PhysicalReadProtectionPolicy::new(one, one);
    let (serving, placement) = fixture::initialize_with_protection(&path, policy);
    let record = append(&serving, placement, 210, b"reopened record");
    let old = serving.records().unwrap();
    let old_root = old.protected_root();
    let old_observer = serving.read_protection_observer();
    assert_eq!(
        serving.close().read_protection().disposition(),
        PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
    );
    let reopened = fixture::open_with_protection(&path, policy);
    let reader = reopened.records().unwrap();
    let new_root = reader.protected_root();
    assert_eq!(old_root.root(), new_root.root());
    assert_ne!(old_root.runtime(), new_root.runtime());
    assert_eq!(
        reopened
            .read_protection_observer()
            .acquisitions_for_root(old_root),
        0
    );
    assert_eq!(old_observer.acquisitions_for_root(new_root), 0);
    assert!(
        matches!(old.open(record, limits(100)), Err(error) if error.denial() == RecordReadDenial::PhysicalWork(worth_store::physical_runtime::RecordReadWorkDenial::RuntimeReleased))
    );
    let mut session = reader.open(record, limits(100)).unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        b"reopened record"
    );
    drop((session, reader, old));
    assert_eq!(old_observer.snapshot().live_acquisitions(), 0);
    fixture::assert_clean_close(reopened);
}

fn borrowed_shutdown(abort: bool) {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("borrowed-shutdown"));
    let expected = fixture::payload(2 * fixture::CHUNK_PAYLOAD_BYTES + 5);
    let record = append(&serving, placement, 206, &expected);
    let observer = serving.read_protection_observer();
    let mut session = serving
        .records()
        .unwrap()
        .open(record, limits(expected.len()))
        .unwrap();
    let chunk = session.next_chunk().unwrap().unwrap();
    let shutdown = if abort {
        serving.abort().read_protection()
    } else {
        serving.close().read_protection()
    };
    assert_eq!(
        shutdown.disposition(),
        PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
    );
    assert_eq!(shutdown.observation().live_acquisitions(), 1);
    assert!(observer.snapshot().revoked());
    assert_eq!(chunk.bytes(), &expected[..fixture::CHUNK_PAYLOAD_BYTES]);
    drop(chunk);
    let before = session.observation();
    assert!(
        matches!(session.next_chunk(), Err(error) if error.kind() == RecordStreamFailureKind::RuntimeReleased)
    );
    assert_eq!(session.observation(), before);
    drop(session);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    assert_eq!(observer.snapshot().releases(), 1);
}

fn limits(bytes: usize) -> RecordReadLimits {
    RecordReadLimits::new(RecordByteLimit::new(bytes as u32).unwrap())
}

fn scan_request() -> RecordScanRequest {
    RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap())
}

fn collect_ids(scan: &mut PhysicalRecordScanSession) -> Vec<PhysicalRecordId> {
    let mut scratch = vec![0; 65_536];
    let mut ids = Vec::new();
    loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            RecordScanOutcome::Batch(batch) => {
                ids.extend(batch.records().iter().map(|record| record.record_id()))
            }
            RecordScanOutcome::Completed(_) => return ids,
        }
    }
}
