//! Native physical publication overlaps one retained managed-run journal original.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn native_publication_cannot_turn_managed_journal_admission_into_no_effect() {
    let directory = tempfile::tempdir().unwrap();
    let opened = crate::open::physical_admission::open(
        directory.path(),
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    let serving = Arc::new(opened.serving);
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeWalAppend,
    );
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x36; 32]))
        .unwrap();
    let prepared = submission
        .prepare_durable_append_with_manifest_capacity_transition(
            RecordAppendBatch::try_from_iter([b"native-original".to_vec()]).unwrap(),
            crate::open::physical_admission::placement(),
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw();
    let worth_proof::TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(
        prepared,
    )) = prepared
    else {
        panic!("native original must be prepared");
    };
    let native = prepared.start();
    if !gate.await_arrival() {
        gate.release();
        panic!("native original must reach its fixed pre-WAL seam");
    }
    let contended = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::PublicationAdmissionContended,
    );
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let managed = ApplicationQueryJournal::new(&serving, "query.obligations");
    // Both inputs remain live. The second call is a different durable original,
    // not a replay or a replacement of the native mutation.
    let overlap = managed.advance(&request, 1, b"managed-retention-original");
    let reached_contention = contended.await_arrival();
    gate.release();
    let native_outcome = native.wait();
    contended.release();
    assert!(
        reached_contention,
        "journal must attempt admission while the native root is owned"
    );
    assert!(
        matches!(native_outcome, PhysicalMutationOutcome::Completed(_)),
        "native original must commit"
    );
    match overlap {
        QueryJournalPublicationOutcome::Pending => {}
        QueryJournalPublicationOutcome::Committed { position: 1, .. } => {}
        QueryJournalPublicationOutcome::ProvenNoEffect { .. } => {
            panic!("overlap manufactured no-effect")
        }
        QueryJournalPublicationOutcome::Refused(cause) => panic!("overlap refused: {cause:?}"),
        QueryJournalPublicationOutcome::Unresolved => panic!("overlap lost the original"),
        QueryJournalPublicationOutcome::Committed { position, .. } => {
            panic!("wrong position: {position}")
        }
    }
    // This is the owning endpoint's progression of the same admitted original.
    loop {
        request.consult().unwrap();
        match managed.advance(&request, 1, b"managed-retention-original") {
            QueryJournalPublicationOutcome::Pending => std::thread::yield_now(),
            QueryJournalPublicationOutcome::Committed { position: 1, .. } => break,
            QueryJournalPublicationOutcome::ProvenNoEffect { .. } => {
                panic!("retained original became no-effect")
            }
            QueryJournalPublicationOutcome::Refused(cause) => {
                panic!("retained original refused: {cause:?}")
            }
            QueryJournalPublicationOutcome::Unresolved => {
                panic!("retained original became unresolved")
            }
            QueryJournalPublicationOutcome::Committed { position, .. } => {
                panic!("wrong position: {position}")
            }
        }
    }
    assert_eq!(
        managed.read_frame(&request, 1, 1024).unwrap().unwrap(),
        b"managed-retention-original"
    );
}
