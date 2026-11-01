//! Real physical publications remain opaque until Native admits the artifact.
use super::*;
use sha2::{Digest, Sha256};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberLocator, RelationalDurableReadProtection,
    RelationalDurableRecoverySource,
};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordPublication, OrderedJournalPublication,
    PhysicalMutationDeadline,
};

fn serial(limit: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(limit)),
        CancellationSource::new().token(),
        None,
    )
}
fn locator(bytes: &[u8]) -> RelationalDurableMemberLocator {
    RelationalDurableMemberLocator {
        sha256: Sha256::digest(bytes).into(),
        length: bytes.len() as u64,
    }
}
fn publish_journal(
    serving: &ServingPhysicalRuntime,
    position: u64,
    bytes: &[u8],
    transition: bool,
) {
    let journal = if transition {
        serving.branch_transitions().unwrap()
    } else {
        serving.canonical_commits().unwrap()
    };
    journal
        .publish(OrderedJournalPublication::new(
            position,
            bytes.to_vec(),
            worth_store_testkit_store::fixture::placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
}
fn publish_member(serving: &ServingPhysicalRuntime, generation: u64, bytes: &[u8]) {
    serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                generation,
                b"opaque Native selection".to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(vec![AnchoredRecordMember::New {
                key: Sha256::digest(bytes).into(),
                bytes: bytes.to_vec(),
            }]),
        )
        .unwrap();
}
#[test]
fn protected_native_cut_reads_original_journals_and_members_after_advance() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(crate::cold_source::test_store::initialize(directory.path()));
    let commit = vec![41; 150_001];
    let transition = b"original transition bytes";
    let member = vec![71; 70_003];
    publish_journal(&serving, 1, &commit, false);
    publish_journal(&serving, 1, transition, true);
    publish_member(&serving, 0, &member);
    let host = serial(8 << 20);
    let request = ExecutionRequest::serial(&host);
    let concrete = StoreColdSource::new(serving.clone());
    let source: Arc<dyn RelationalDurableColdSource> = concrete.clone();
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    assert_eq!(
        cut.bounds(),
        Bounds {
            canonical: PatchStreamPosition(1),
            transitions: RelationalBranchTransitionSequence(1)
        }
    );
    publish_journal(&serving, 2, b"later canonical", false);
    publish_journal(&serving, 2, b"later transition", true);
    publish_member(&serving, 1, b"latest selected member");
    // The live cache is deliberately unrelated; frozen N owns its verified table.
    assert!(concrete.member_locators().is_empty());
    let bytes = cut
        .read_commit_owned_at(
            &source,
            &request,
            PatchStreamPosition(1),
            commit.len() as u64,
        )
        .unwrap();
    assert_eq!(bytes.bytes(), commit);
    assert_eq!(
        cut.read_transition_owned_at(
            &source,
            &request,
            RelationalBranchTransitionSequence(1),
            100
        )
        .unwrap()
        .bytes(),
        transition
    );
    assert_eq!(
        cut.read_member_owned_at(&source, &request, locator(&member), member.len() as u64)
            .unwrap()
            .bytes(),
        member
    );
    assert!(matches!(
        cut.read_commit_owned_at(&source, &request, PatchStreamPosition(2), 100),
        Err(Refusal::Bounds)
    ));
    assert!(matches!(
        cut.read_transition_owned_at(
            &source,
            &request,
            RelationalBranchTransitionSequence(2),
            100
        ),
        Err(Refusal::Bounds)
    ));
    assert!(matches!(
        cut.read_member_owned_at(&source, &request, locator(b"latest selected member"), 100),
        Err(Refusal::Storage(Cause::MissingMember))
    ));
    assert_eq!(
        serving
            .anchored_records()
            .unwrap()
            .read_member(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                // Only the latest publication's ordinary read can resolve its member.
                serving
                    .anchored_records()
                    .unwrap()
                    .read(AnchoredRecordName::new("home.checkpoint").unwrap(), 1 << 20)
                    .unwrap()
                    .unwrap()
                    .members()[0],
                100
            )
            .unwrap(),
        b"latest selected member"
    );
    drop(cut);
    drop(source);
    drop(concrete);
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        commit.len() as u64
    );
    let clone = bytes.clone();
    drop(bytes);
    assert_eq!(clone.bytes(), commit);
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}
#[test]
fn protected_native_cut_refuses_foreign_limits_and_actual_buffer_growth() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(crate::cold_source::test_store::initialize(directory.path()));
    let payload = vec![9u8; 150_001];
    publish_journal(&serving, 1, &payload, false);
    let host = serial(8 << 20);
    let request = ExecutionRequest::serial(&host);
    let concrete = StoreColdSource::new(serving.clone());
    let source: Arc<dyn RelationalDurableColdSource> = concrete.clone();
    let other_concrete = StoreColdSource::new(serving.clone());
    let other: Arc<dyn RelationalDurableColdSource> = other_concrete.clone();
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    let io = serving.media_counters();
    assert!(matches!(
        cut.read_commit_owned_at(&other, &request, PatchStreamPosition(1), 1 << 20),
        Err(Refusal::ForeignSource)
    ));
    assert_eq!(other_concrete.counters().commit_reads, 0);
    assert_eq!(serving.media_counters(), io);
    assert!(matches!(
        cut.read_commit_owned_at(&source, &request, PatchStreamPosition(1), 1),
        Err(Refusal::Storage(Cause::LimitExceeded {
            length: 150_001,
            limit: 1
        }))
    ));
    let small = serial(payload.len() as u64 - 1);
    let original = ExecutionRequest::serial(&small);
    let entered = concrete.counters().commit_reads;
    let refusal = cut
        .read_commit_owned_at(&source, &original, PatchStreamPosition(1), 1 << 20)
        .unwrap_err();
    assert!(concrete.counters().commit_reads > entered);
    assert!(
        matches!(
            refusal,
            Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(
                worth_execution::WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(
                        worth_execution::MemoryLimitDenial::Capacity {
                            requested: 150_001,
                            ..
                        }
                    )
                )
            ))
        ),
        "actual payload reservation, not a controller-entry refusal: {refusal:?}"
    );
    assert_eq!(small.memory().unwrap().charged_memory_bytes(), 0);
    let output = cut
        .read_commit_owned_at(&source, &request, PatchStreamPosition(1), 1 << 20)
        .unwrap();
    assert_eq!(output.bytes(), payload);
    drop(output);
    drop(cut);
    drop(source);
    drop(concrete);
    drop(other);
    drop(other_concrete);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}
#[test]
fn protected_native_source_preserves_malformed_original_for_native_frame_refusal() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(crate::cold_source::test_store::initialize(directory.path()));
    let malformed = b"actual Store accepts opaque bytes; Native refuses this frame";
    publish_journal(&serving, 1, malformed, false);
    let host = serial(8 << 20);
    let request = ExecutionRequest::serial(&host);
    let concrete = StoreColdSource::new(serving.clone());
    let source: Arc<dyn RelationalDurableColdSource> = concrete.clone();
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    let output = cut
        .read_commit_owned_at(&source, &request, PatchStreamPosition(1), 100)
        .unwrap();
    assert_eq!(output.bytes(), malformed);
    drop(output);
    let entered = concrete.counters().commit_reads;
    assert!(matches!(
        RelationalDurableRecoverySource::from_protected_source(
            &request,
            None,
            source.clone(),
            &cut
        ),
        Err(Refusal::CanonicalFrame(_))
    ));
    assert!(concrete.counters().commit_reads > entered);
    drop(cut);
    drop(source);
    drop(concrete);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}
