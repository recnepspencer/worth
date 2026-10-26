//! Journal-body cells are selected independently of readable commit metadata.
use super::cold_envelopes::EnvelopeSurface;
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::history::BoundedCanonicalCommitPatchDenial;
use crate::publication::patch::data::{PatchStreamReadErrorClass, PatchStreamRequest};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[derive(Debug, Clone, Copy)]
enum EnvelopeDoor {
    Surface(EnvelopeSurface),
    Patches,
    Stream,
}
impl EnvelopeDoor {
    fn read(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::history::data::RelationalCommitReceipt,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) -> Result<usize, Execution> {
        match self {
            Self::Surface(surface) => surface.read(execution, runtime, receipt),
            Self::Patches => runtime
                .history()
                .bounded_canonical_commit_patches(
                    execution,
                    receipt.commit_id,
                    std::num::NonZeroUsize::new(32).unwrap(),
                )
                .map(|patches| patches.len())
                .map_err(|denial| match denial {
                    BoundedCanonicalCommitPatchDenial::ReadRequest(cause) => cause,
                    other => panic!("producer patch bound fits: {other:?}"),
                }),
            Self::Stream => runtime
                .publication()
                .read_patch_stream(
                    execution,
                    PatchStreamRequest {
                        after_position: Some(crate::publication::patch::data::PatchStreamPosition(
                            position.0 - 1,
                        )),
                        max_commits: 1,
                    },
                )
                .map(|batch| batch.patches.len())
                .map_err(|denial| match denial.class {
                    PatchStreamReadErrorClass::ReadRequest(cause) => cause,
                    other => panic!("producer stream position is admitted: {other:?}"),
                }),
        }
    }
}
#[test]
fn residency_failure_matrix_envelope_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in EnvelopeSurface::ALL
        .map(EnvelopeDoor::Surface)
        .into_iter()
        .chain([EnvelopeDoor::Patches, EnvelopeDoor::Stream])
    {
        for cause in FAILURES {
            let runtime = || match door {
                EnvelopeDoor::Surface(EnvelopeSurface::CompiledArtifact) => {
                    RelationalRuntimeApi::builder()
                        .profile(RelationalRuntimeProfile::ChipSimulation)
                        .schema_registry(test_schema_registry())
                        .build_durable(TestDurableMedium::new().attachment())
                }
                _ => durable_runtime_with_test_schema(),
            };
            let (recovery, source, _) = fixture_from_runtime(execution, runtime(), false, 16);
            source.damaged.store(false, Ordering::Relaxed);
            let position = match door {
                EnvelopeDoor::Surface(
                    EnvelopeSurface::RecentInspection | EnvelopeSurface::RecentBranchInspection,
                ) => *source.commits.keys().rev().nth(1).unwrap(),
                _ => *source.commits.keys().nth(3).unwrap(),
            };
            let mut runtime = runtime();
            let plan = runtime
                .durability()
                .recovery_plan_from_source(
                    execution,
                    recovery,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap();
            runtime
                .durability_recovery()
                .recover(execution, plan)
                .unwrap();
            let receipt = runtime
                .history()
                .immutable_commit_receipt_at_patch_stream_position(position)
                .unwrap();
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&runtime).offered().len();
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
                "{door:?} x {cause:?}"
            );
            source.failures.lock().unwrap().clear();
            let reads = source.commit_reads.lock().unwrap().len();
            if cause.is_proven_damage() {
                assert_eq!(
                    door.read(&runtime, execution, &receipt, position),
                    Err(expected)
                );
                assert_eq!(source.commit_reads.lock().unwrap().len(), reads);
            } else {
                assert!(
                    door.read(&runtime, execution, &receipt, position).unwrap() > 0,
                    "retry returns genuine body contents"
                );
                assert!(source.commit_reads.lock().unwrap().len() > reads);
            }
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
        }
    }
}
