//! Real Memory publication owns one original frame; tuple observations stay scalar.
use super::*;
use crate::durability::canonical_commit_artifact::verified_frame::VerifiedCanonicalCommitFrame;
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;
use sha2::{Digest, Sha256};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn memory_original_frame_survives_export_checkpoint_and_last_alias() {
    let budget = SerialMemoryBudget::new(Some(8 << 20));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(&execution, &runtime, "original-memory-frame");
    let cell = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let original = {
        let state = cell.state.lock().unwrap();
        let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        else {
            panic!("real Memory publication owns original bytes")
        };
        bytes.clone()
    };
    assert_eq!(
        original.charged_payload_bytes(),
        Some(original.len() as u64)
    );
    // The escaped frame is a public byte owner: its payload and its control.
    let bill = retained_byte_owner_bill(original.bytes());
    let positioned = cell.demand_positioned(&execution, 1 << 30).unwrap();
    // Independent existing encoder is a fixture reference, outside the export claim.
    let reference = VerifiedCanonicalCommitFrame::reference_encode(
        &positioned,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    assert_eq!(reference.as_bytes(), original.bytes());
    let before = budget.charged_memory_bytes();
    let exported = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(exported.as_bytes().as_ptr(), original.bytes().as_ptr());
    assert_eq!(
        budget.charged_memory_bytes(),
        before,
        "export verifies/shares; it allocates no replacement frame"
    );
    let native = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    assert!(native
        .bytes()
        .starts_with(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC));
    let again = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(again.as_bytes().as_ptr(), original.bytes().as_ptr());
    release_test_commit_snapshot(&runtime, &result);
    drop(native);
    drop(positioned);
    drop(reference);
    drop(result);
    drop(cell);
    drop(runtime);
    drop(original);
    drop(again);
    assert_eq!(
        budget.charged_memory_bytes(),
        bill,
        "only the escaped original frame remains"
    );
    let clone = exported.clone();
    drop(exported);
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(clone);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn memory_frame_preserves_original_payload_and_scalar_tuple_observations() {
    let host = crate::execution::test_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let result = create_entity_outcome(&execution, &runtime, "tuple-observations");
    let artifact = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap();
    let envelope = artifact.resident_envelope().unwrap();
    let bytes = envelope.encode_authoritative_payload().unwrap();
    assert_eq!(artifact.canonical_payload_bytes(), bytes.len() as u64);
    assert_eq!(
        artifact.canonical_payload_digest().unwrap(),
        <[u8; 32]>::from(Sha256::digest(&bytes))
    );
    assert!(artifact.envelope_residency().payload_storage_bytes() > bytes.len() as u64);
    let cell = artifact.envelope_residency();
    let original =
        crate::durability::canonical_commit_artifact::retained_frame_reads::export_artifact(
            &cell, &execution,
        )
        .unwrap();
    assert_ne!(
        original.payload_digest().as_bytes(),
        &artifact.canonical_payload_digest().unwrap()
    );
    let old = OldCell {
        state: Mutex::new(OldState {
            authored_writes: None,
            backing: OldBacking::Memory(envelope),
            position: None,
            damage: None,
            payload_digest: None,
            canonical_payload: None,
            lineage: None,
        }),
    };
    let OldState {
        authored_writes,
        backing,
        position,
        damage,
        payload_digest,
        canonical_payload,
        lineage,
    } = old.state.into_inner().unwrap();
    if let OldBacking::Memory(envelope) = backing {
        drop(envelope);
    }
    drop((
        authored_writes,
        position,
        damage,
        payload_digest,
        canonical_payload,
        lineage,
    ));
    eprintln!(
        "old/new backing {}/{}, state {}/{}, cell {}/{}; immutable owner {}",
        std::mem::size_of::<OldBacking>(),
        std::mem::size_of::<EnvelopeBacking>(),
        std::mem::size_of::<OldState>(),
        std::mem::size_of::<EnvelopeState>(),
        std::mem::size_of::<OldCell>(),
        std::mem::size_of::<RelationalCommitEnvelopeResidency>(),
        std::mem::size_of::<worth_execution::ExecutionImmutableBytes>()
    );
    assert_eq!(
        std::mem::size_of::<EnvelopeBacking>(),
        std::mem::size_of::<OldBacking>() + std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::size_of::<RelationalCommitEnvelopeResidency>(),
        std::mem::size_of::<OldCell>() + std::mem::size_of::<usize>()
    );
    release_test_commit_snapshot(&runtime, &result);
}

// The prior concrete layouts, preserving actual field types and variant order.
enum OldBacking {
    Memory(Arc<CanonicalCommitEnvelope>),
    #[expect(
        dead_code,
        reason = "never built here; it gives the prior layout its actual size"
    )]
    Journal {
        entry: Arc<RelationalCommitIndexEntry>,
        source: Arc<dyn RelationalDurableColdSource>,
        unsettled: Option<Arc<CanonicalCommitEnvelope>>,
    },
}
struct OldState {
    authored_writes:
        Option<crate::history::access::relevant_writes::generation::AuthoredWriteGeneration>,
    backing: OldBacking,
    position: Option<RetainedCanonicalPosition>,
    damage: Option<RelationalStorageUnavailable>,
    payload_digest: Option<[u8; 32]>,
    canonical_payload: Option<Arc<[u8]>>,
    lineage: Option<crate::lineage::data::LineageCheckpointDigestBasis>,
}
struct OldCell {
    state: Mutex<OldState>,
}

// Fixture adaptation (C2.4/C4): restored from 86e84dc5cd:memory_publication_tests.rs
// using case-owned `OwnedTestDurableMedium` (no runtime_attachment).
#[test]
fn journal_publication_keeps_journal_placement_without_memory_frame() {
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let result = create_entity_outcome(&execution, &runtime, "unchanged-journal-frame");
    let cell = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let (entry, source) = cell.journal_placement().unwrap();
    assert!(entry.same_owner(&cell.journal_placement().unwrap().0));
    assert!(cell.resident_payload_bytes().is_none());
    assert_eq!(cell.payload_storage_bytes(), 0);
    let original = medium.latest_committed_artifact().unwrap();
    let exported =
        crate::durability::canonical_commit_artifact::retained_frame_reads::export_artifact(
            &cell, &execution,
        )
        .unwrap();
    assert_eq!(exported.as_bytes(), original.as_bytes());
    assert!(Arc::ptr_eq(
        &source,
        &runtime.durability.cold_source().unwrap()
    ));
    release_test_commit_snapshot(&runtime, &result);
}

#[test]
fn recovered_nonhead_tail_envelopes_release_demanded_journal_body_on_drop() {
    let host = crate::execution::test_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let entity = create_entity(&execution, &runtime, "tail-body-0");
    for n in 1..4 {
        update_entity_and_release_snapshot(&execution, &runtime, entity, &format!("tail-body-{n}"));
    }
    let mut recovered = durable_runtime_for_medium(&medium);
    recover_runtime_from_medium(&execution, &mut recovered, &medium);
    recovered.history.reclaim_retired_branch_roots(1024);
    let receipt = recovered
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(2))
        .unwrap();
    let body = recovered
        .history()
        .demand_commit_envelope(&execution, receipt.commit_id)
        .unwrap()
        .unwrap();
    let weak = Arc::downgrade(&body);
    drop(body);
    assert!(
        weak.upgrade().is_none(),
        "replay must not retain a recovered nonhead journal-backed body after drop"
    );
    assert_eq!(
        recovered
            .history()
            .demand_commit_envelope(&execution, receipt.commit_id)
            .unwrap()
            .unwrap()
            .commit,
        receipt
    );
}
