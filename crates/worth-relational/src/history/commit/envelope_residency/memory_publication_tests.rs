//! Real Memory publication owns one original frame; tuple observations stay scalar.
use super::*;
use crate::tests::support::*;
use sha2::{Digest, Sha256};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}
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
    let bill = original.len() as u64;
    assert_eq!(original.charged_payload_bytes(), Some(bill));
    let positioned = cell.demand_positioned(&execution, 1 << 30).unwrap();
    // Independent existing encoder is a fixture reference, outside the export claim.
    let reference = RelationalCanonicalCommitArtifact::from_positioned(
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
fn memory_frame_quota_and_cancel_refuse_before_position_or_head() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("quota-memory-frame"),
            setup_request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&setup_request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&setup_request, proposal)
        .unwrap();
    let (_, _, next) = candidate.change_summary_basis_and_roots().unwrap();
    let envelope = next.canonical_envelope().unwrap();
    #[derive(serde::Serialize)]
    struct Reference<'a> {
        position: crate::publication::patch::data::PatchStreamPosition,
        canonical: crate::history::data::AuthoritativeArtifactEnvelopeRef<'a>,
        original_inputs:
            Option<&'a crate::durability::source_initial_inputs::recorded::RecordedCanonicalInputs>,
    }
    let position = runtime
        .history
        .latest_recorded_patch_position()
        .map_or(1, |position| position.0 + 1);
    let reference = rmp_serde::to_vec_named(&Reference {
        position: crate::publication::patch::data::PatchStreamPosition(position),
        canonical: crate::history::data::AuthoritativeArtifactEnvelopeRef::new(envelope),
        original_inputs: envelope.recorded_original_inputs(),
    })
    .unwrap();
    let bill = (8 + 2 + 8 + 8 + 32 + 8 + reference.len()) as u64;
    let controller = controller_memory();
    assert!(bill > controller);
    let counters = runtime.patch_position_reservation_counters();
    let before_head = runtime.history.latest_recorded_patch_position();
    let limited = SerialMemoryBudget::new(Some(controller.checked_add(bill).unwrap() - 1));
    let host = SerialRequest::from_memory(limited.clone(), CancellationSource::new().token(), None);
    let original = ExecutionRequest::serial(&host);
    let outcome = runtime
        .publication_port()
        .compare_and_publish(&original, candidate);
    let crate::mvcc::RelationalPublicationOutcome::RequestStopped(
        worth_execution::WorkCeilingDenial::Admission(
            worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity { requested, .. },
            ),
        ),
    ) = outcome
    else {
        panic!("the actual frame reservation must refuse")
    };
    assert_eq!(
        requested, bill,
        "independent reference frame bytes, not the controller or preparation"
    );
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(
        runtime.history.latest_recorded_patch_position(),
        before_head
    );
    runtime.compare_current_exact(&basis).unwrap();
    assert_eq!(limited.charged_memory_bytes(), 0);
    drop(next);
    drop(basis);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("cancel-memory-frame"),
            setup_request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(&setup_request, &runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(&setup_request, proposal)
        .unwrap();
    let cancel = CancellationSource::new();
    let cancelled = SerialMemoryBudget::new(Some(8 << 20));
    let host = SerialRequest::from_memory(cancelled.clone(), cancel.token(), None);
    let original = ExecutionRequest::serial(&host);
    cancel.cancel();
    assert!(matches!(
        runtime
            .publication_port()
            .compare_and_publish(&original, candidate),
        crate::mvcc::RelationalPublicationOutcome::RequestStopped(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments
    );
    assert_eq!(
        runtime.history.latest_recorded_patch_position(),
        before_head
    );
    assert_eq!(cancelled.charged_memory_bytes(), 0);
    let result = create_entity_outcome(&setup_request, &runtime, "caller-after-refusals");
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        counters.assignments + 1
    );
    release_test_commit_snapshot(&runtime, &result);
}
// The prior concrete layouts, preserving actual field types and variant order.
enum OldBacking {
    Memory(Arc<CanonicalCommitEnvelope>),
    Journal {
        entry: Arc<RelationalCommitIndexEntry>,
        source: Arc<dyn RelationalDurableColdSource>,
        unsettled: Option<Arc<CanonicalCommitEnvelope>>,
    },
}
struct OldState {
    authored_writes: Option<crate::runtime::AuthoredWriteGeneration>,
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
#[test]
fn memory_frame_layout_preserves_scalar_tuple_observations() {
    let host = crate::test_execution_policy::serial_request();
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
    let original = cell.export_artifact(&execution).unwrap();
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
#[test]
fn journal_publication_keeps_journal_placement_without_memory_frame() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let result = create_entity_outcome(&execution, &runtime, "unchanged-journal-frame");
    let cell = runtime
        .history
        .commit_artifact(result.commit.commit_id)
        .unwrap()
        .envelope_residency();
    let (entry, source) = cell.journal_placement().unwrap();
    // Descriptive OLD-layout value only, outside funding/no-copy claims.
    // Its exact historical Arc<Entry> type remains independent of the new owner.
    let old = OldBacking::Journal {
        entry: Arc::new(entry.as_ref().clone()),
        source: source.clone(),
        unsettled: None,
    };
    if let OldBacking::Journal {
        entry,
        source,
        unsettled,
    } = old
    {
        drop((entry, source, unsettled));
    }
    assert!(entry.same_owner(&cell.journal_placement().unwrap().0));
    assert!(cell.resident_payload_bytes().is_none());
    assert_eq!(cell.payload_storage_bytes(), 0);
    let original = attached_test_medium(&runtime)
        .artifact_bytes(entry.position)
        .unwrap();
    let exported = cell.export_artifact(&execution).unwrap();
    assert_eq!(exported.as_bytes(), original.as_ref());
    assert!(Arc::ptr_eq(
        &source,
        &runtime.durability.cold_source().unwrap()
    ));
    release_test_commit_snapshot(&runtime, &result);
}
