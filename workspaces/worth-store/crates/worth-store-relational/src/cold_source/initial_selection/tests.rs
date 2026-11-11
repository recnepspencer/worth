//! Actual selected Store reads, admission refusal and original funding custody.
use super::*;
use crate::cold_source::test_store;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordPublication, AnchoredRecordPublished,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};

fn publish(
    serving: &ServingPhysicalRuntime,
    generation: u64,
    members: Vec<AnchoredRecordMember>,
) -> AnchoredRecordPublished {
    serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                generation,
                b"opaque selecting payload".to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(members),
        )
        .unwrap()
}
fn selected(
    serving: &ServingPhysicalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> AnchoredRecordReadMembership {
    let reader = serving.try_records_for_checkpoint(request).unwrap();
    let records = serving.anchored_records().unwrap();
    let mut stream = records
        .open_payload_from(
            &reader,
            request,
            AnchoredRecordName::new("home.checkpoint").unwrap(),
            1 << 20,
        )
        .unwrap()
        .unwrap();
    let mut scratch = [0; 31];
    while stream.read_next(&mut scratch).unwrap() != 0 {}
    stream
        .into_verified_membership(reader)
        .unwrap_or_else(|_| panic!("verified original read"))
}
fn member(bytes: &[u8]) -> AnchoredRecordMember {
    AnchoredRecordMember::New {
        key: Sha256::digest(bytes).into(),
        bytes: bytes.to_vec(),
    }
}
fn locator(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}

#[test]
fn selected_read_keeps_original_members_after_replacement_and_last_listed_placement() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let bytes = vec![71; 150_001];
    let first = publish(&serving, 0, vec![member(&bytes)]);
    let second = publish(
        &serving,
        1,
        vec![
            AnchoredRecordMember::Carried {
                locator: first.members()[0],
            },
            member(&bytes),
        ],
    );
    assert_ne!(second.members()[0], second.members()[1]);
    let read = selected(&serving, &request);
    let source = StoreColdSource::new(serving.clone());
    let charge_before = serial.memory().unwrap().charged_memory_bytes();
    source
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    let charge_after = serial.memory().unwrap().charged_memory_bytes();
    assert!(
        charge_after > charge_before,
        "actual placement backing is admitted before growth"
    );
    assert_eq!(
        source
            .members
            .read()
            .unwrap()
            .entries
            .get(&locator(&bytes))
            .unwrap()
            .index,
        1
    );
    let latest = publish(&serving, 2, vec![member(b"replacement")]);
    assert_eq!(source.read_member(locator(&bytes), 1 << 20).unwrap(), bytes);
    assert!(matches!(
        source.select_members(latest.membership()),
        Err(Denial::DifferentSelection)
    ));
    source.select_members(second.membership()).unwrap();
    drop(read);
    assert_eq!(
        serial.memory().unwrap().charged_memory_bytes(),
        charge_after,
        "the installed map retains the same original physical backing and its grants"
    );
    assert_eq!(source.read_member(locator(&bytes), 1 << 20).unwrap(), bytes);
    let ordinary = StoreColdSource::new(serving.clone());
    ordinary.select_members(second.membership()).unwrap();
    assert_eq!(
        ordinary.read_member(locator(&bytes), 1 << 20),
        Err(crate::cold_source::Cause::MissingMember)
    );
    drop(ordinary);
    drop(source);
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}

#[test]
fn zero_budget_refuses_before_install_and_exact_controlled_retry_does_not_grow() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    publish(&serving, 0, vec![member(b"funded map")]);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let zero = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationSource::new().token(),
        None,
    );
    let empty_request = ExecutionRequest::serial(&zero);
    let read = selected(&serving, &request);
    let source = StoreColdSource::new(serving.clone());
    let io_before = serving.media_counters();
    assert!(matches!(
        source.select_initial_members_from_read(&read, &empty_request),
        Err(Denial::Allocation(_))
    ));
    assert_eq!(serving.media_counters(), io_before);
    assert_eq!(
        source.read_member(locator(b"funded map"), 1 << 20),
        Err(crate::cold_source::Cause::MissingMember)
    );
    assert_eq!(zero.memory().unwrap().charged_memory_bytes(), 0);
    source
        .select_initial_members_from_read(&read, &request)
        .unwrap();
    let charged = serial.memory().unwrap().charged_memory_bytes();
    source
        .select_initial_members_from_read(&read, &empty_request)
        .unwrap();
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), charged);
    assert_eq!(zero.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        source.read_member(locator(b"funded map"), 1 << 20).unwrap(),
        b"funded map"
    );
    drop(read);
    drop(source);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}

#[test]
fn foreign_busy_stopped_and_ordinary_selection_cannot_be_promoted_to_controlled() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let foreign = Arc::new(test_store::initialize(foreign_directory.path()));
    let first = publish(&serving, 0, vec![member(b"original")]);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 << 20)),
        cancellation.token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let read = selected(&serving, &request);
    let source = StoreColdSource::new(serving.clone());
    let other = StoreColdSource::new(foreign.clone());
    let io_before = foreign.media_counters();
    assert!(matches!(
        other.select_initial_members_from_read(&read, &request),
        Err(Denial::Read(_))
    ));
    assert_eq!(foreign.media_counters(), io_before);
    let held = source.members.write().unwrap();
    assert!(matches!(
        source.select_initial_members_from_read(&read, &request),
        Err(Denial::Busy)
    ));
    drop(held);
    source
        .custody
        .try_select(|| {
            assert!(matches!(
                source.select_initial_members_from_read(&read, &request),
                Err(Denial::Busy)
            ));
        })
        .unwrap();
    source.select_members(first.membership()).unwrap();
    assert!(matches!(
        source.select_initial_members_from_read(&read, &request),
        Err(Denial::DifferentSelection)
    ));
    let fresh = StoreColdSource::new(serving.clone());
    cancellation.cancel();
    assert!(matches!(
        fresh.select_initial_members_from_read(&read, &request),
        Err(Denial::Execution(_))
    ));
    assert_eq!(
        fresh.read_member(locator(b"original"), 1 << 20),
        Err(crate::cold_source::Cause::MissingMember)
    );
    drop(read);
    drop(fresh);
    drop(source);
    drop(other);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
    let _foreign_closed = Arc::try_unwrap(foreign).ok().unwrap().close();
}
