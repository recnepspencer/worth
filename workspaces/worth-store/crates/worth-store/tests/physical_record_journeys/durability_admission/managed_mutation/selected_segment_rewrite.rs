use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationProvenNoEffectCause, PhysicalMutationRequest,
    RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_backend::MediaOperationRole;
use super::super::independent_wal_oracle::produced_rewrite_payloads;
use super::*;

#[test]
fn selected_segment_rewrite_preserves_record_bytes_and_writes_rewrite_redo() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = b"selected-segment-rewrite";
    let appended = completed(prepare(&serving, placement, [31; 32], payload).execute());
    let identity = appended.persisted_records()[0];
    let before = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    let rewritten = completed(prepare_rewrite(&serving, placement, [32; 32]).execute());
    assert_eq!(rewritten.persisted_records(), &[identity]);
    assert!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation()
            > before
    );
    let record = rewritten.into_acknowledgment().record_ids().next().unwrap();
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(payload.len() as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; payload.len()];
    let mut filled = 0;
    while filled < bytes.len() {
        let count = session.read_next(&mut bytes[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    assert_eq!(bytes, payload);
    let rewrites = produced_rewrite_payloads(&root);
    assert_eq!(rewrites.len(), 1);
    assert_eq!(rewrites[0].source_length, rewrites[0].destination_length);
    assert_eq!(u64::from(rewrites[0].source_length), rewrites[0].candidate_bytes);
    assert!(rewrites[0].resulting_root_generation > rewrites[0].source_root_generation);
    serving.close();
}

#[test]
fn reopened_store_keeps_the_published_page_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [42; 32], b"reopen-charge").execute());
    completed(prepare(&serving, placement, [44; 32], b"reopen-charge-again").execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the sealed publication charge"
    );
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match prepare(&serving, placement, [43; 32], b"reopen-over-growth").execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("reopen must not restore growth already consumed by the published page")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("reopen growth denial must prove no effect before WAL effects")
        }
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn append_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match prepare(&serving, placement, [41; 32], b"growth-append").execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("one-byte-over growth must deny the append before WAL effects")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("one-byte-over growth must prove no effect before WAL effects")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn append_is_denied_when_usable_growth_is_only_the_data_page() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    serving.certification_limit_candidate_growth_bytes(page_bytes);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match prepare(&serving, placement, [46; 32], b"wal-root-growth").execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("WAL and root metadata must deny an append that only budgeted its data page")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("WAL and root metadata must prove no effect before WAL effects")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn a_completed_rewrite_retries_as_the_same_request() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [44; 32], b"retry-source").execute());
    let key = serving
        .record_submission()
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([45; 32]))
        .unwrap();
    let request = PhysicalMutationRequest::platform_durable(
        key.clone(),
        PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    );
    match serving
        .record_submission()
        .rewrite_selected_inline_segment(placement, request)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(
            prepared,
        )) => {
            completed(prepared.execute());
        }
        _ => panic!("the first rewrite must prepare"),
    }
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let retry = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    );
    match serving
        .record_submission()
        .rewrite_selected_inline_segment(placement, retry)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(_)) => {
        }
        worth_proof::TransitionOutcome::Denied(_) => {
            panic!("the same rewrite request must return its completed fate")
        }
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("the same rewrite request must not start a second publication")
        }
        worth_proof::TransitionOutcome::Deferred(_)
        | worth_proof::TransitionOutcome::Stale(_)
        | worth_proof::TransitionOutcome::RebindRequired(_)
        | worth_proof::TransitionOutcome::Failed(_) => {
            panic!("the same rewrite request must return its completed fate")
        }
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn selected_segment_rewrite_is_denied_one_byte_over_usable_growth() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [39; 32], b"growth-source").execute());
    serving.certification_limit_candidate_growth_bytes(page_bytes.saturating_sub(1));
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match prepare_rewrite(&serving, placement, [40; 32]).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
            );
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("one-byte-over growth must deny the rewrite before WAL effects")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("one-byte-over growth must prove no effect before WAL effects")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn a_published_predecessor_invalidates_a_prepared_rewrite() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [33; 32], b"rewrite-source").execute());
    let rewrite = prepare_rewrite(&serving, placement, [34; 32]);
    completed(prepare(&serving, placement, [35; 32], b"published-predecessor").execute());
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match rewrite.execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(fate.cause(), PhysicalMutationProvenNoEffectCause::SourceChanged);
        }
        PhysicalMutationOutcome::Completed(_) => {
            panic!("a stale rewrite must not publish")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("a stale rewrite must be refused before its WAL effect")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

#[test]
fn a_wal_durable_append_blocks_rewrite_without_a_second_reservation() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [36; 32], b"published-tail").execute());
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterWalDurability,
    );
    let append = prepare(&serving, placement, [37; 32], b"wal-durable-append").start();
    assert!(gate.await_arrival());
    assert_eq!(serving.certification_pending_publication_count(), 1);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match prepare_rewrite(&serving, placement, [38; 32]).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(fate.cause(), PhysicalMutationProvenNoEffectCause::ScopeConflict);
        }
        PhysicalMutationOutcome::Completed(_) => panic!("rewrite must not pass a pending append"),
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("rewrite must be refused before its WAL effect")
        }
    }
    assert_eq!(serving.certification_pending_publication_count(), 1);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    gate.release();
    append.wait();
    assert_eq!(serving.certification_pending_publication_count(), 0);
    serving.close();
}

pub(super) fn prepare_rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            placement,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(
            prepared,
        )) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("selected segment rewrite preparation must return a prepared mutation")
        }
        worth_proof::TransitionOutcome::Denied(_) => {
            panic!("selected segment rewrite preparation was denied")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("selected segment rewrite preparation was deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("selected segment rewrite preparation was stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("selected segment rewrite preparation required a rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("selected segment rewrite preparation failed")
        }
    }
}

fn completed(
    outcome: PhysicalMutationOutcome,
) -> worth_store::physical_runtime::CompletedPhysicalMutation {
    match outcome {
        PhysicalMutationOutcome::Completed(completed) => completed,
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite journey proved no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite journey became indeterminate at {:?}", fate.stage())
        }
    }
}

