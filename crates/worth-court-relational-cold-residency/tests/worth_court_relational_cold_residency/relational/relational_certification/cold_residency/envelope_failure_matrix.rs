use super::{
    envelope_doors::EnvelopeDoor, envelope_source::reopened, envelope_surfaces::EnvelopeSurface,
    lineage_event_matrix::FAILURES,
};
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    storage::{RelationalStorageSubject, RelationalStorageUnavailable},
    transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn residency_failure_matrix_envelope_doors_preserve_every_failure_class() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for tail_only in [false, true] {
        for door in EnvelopeSurface::ALL
            .map(EnvelopeDoor::Surface)
            .into_iter()
            .chain([
                EnvelopeDoor::Patches,
                EnvelopeDoor::Stream,
                EnvelopeDoor::RetainedExport,
                EnvelopeDoor::EmbeddedCheckpoint,
                EnvelopeDoor::MemoryCheckpoint,
                EnvelopeDoor::ReplayEnvelope,
                EnvelopeDoor::ReplayExecution,
                EnvelopeDoor::ChangeReceipt,
            ])
        {
            for (cause, sticky) in FAILURES {
                let profile = if matches!(
                    door,
                    EnvelopeDoor::Surface(EnvelopeSurface::CompiledArtifact)
                ) {
                    RelationalRuntimeProfile::ChipSimulation
                } else {
                    RelationalRuntimeProfile::CertificationCore
                };
                let fixture = reopened(execution, 16, tail_only, profile);
                let runtime = fixture.runtime;
                let source = fixture.source;
                let medium = fixture.medium;
                let position = match door {
                    EnvelopeDoor::Surface(
                        EnvelopeSurface::RecentInspection | EnvelopeSurface::RecentBranchInspection,
                    ) => *source.commits.keys().rev().nth(1).unwrap(),
                    _ => *source.commits.keys().nth(3).unwrap(),
                };
                runtime.run_branch_root_reclamation_pass();
                let receipt = runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(position)
                    .unwrap();
                let frontier = runtime.history().latest_patch_stream_position();
                let offered = medium.offered().len();
                source
                    .failures
                    .lock()
                    .unwrap()
                    .insert(ColdRead::Commit(position), cause);
                let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(position),
                    cause,
                });
                assert_eq!(
                    door.read(&runtime, execution, &receipt, position),
                    Err(expected),
                    "{door:?} x {cause:?}, tail-only {tail_only}"
                );
                source.failures.lock().unwrap().clear();
                let reads = source
                    .commit_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|read| **read == position)
                    .count();
                if sticky {
                    assert_eq!(
                        runtime
                            .replay()
                            .canonical_commit_envelope(execution, receipt.commit_id),
                        Err(expected),
                        "body reads must share export/capture damage"
                    );
                    assert_eq!(
                        door.read(&runtime, execution, &receipt, position),
                        Err(expected)
                    );
                    assert_eq!(
                        source
                            .commit_reads
                            .lock()
                            .unwrap()
                            .iter()
                            .filter(|read| **read == position)
                            .count(),
                        reads
                    );
                } else {
                    assert!(
                        door.read(&runtime, execution, &receipt, position).unwrap() > 0,
                        "retry returns genuine body contents"
                    );
                    assert!(
                        source
                            .commit_reads
                            .lock()
                            .unwrap()
                            .iter()
                            .filter(|read| **read == position)
                            .count()
                            > reads
                    );
                }
                assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                assert_eq!(medium.offered().len(), offered);
            }
        }
    }
}
