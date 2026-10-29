use super::*;

#[test]
fn residency_failure_matrix_envelope_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
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
                let runtime = || match door {
                    EnvelopeDoor::Surface(EnvelopeSurface::CompiledArtifact) => {
                        RelationalRuntimeApi::builder()
                            .profile(RelationalRuntimeProfile::ChipSimulation)
                            .schema_registry(test_schema_registry())
                            .build_durable(TestDurableMedium::new().attachment())
                            .expect("valid native runtime configuration")
                    }
                    _ => durable_runtime_with_test_schema(),
                };
                let (mut recovery, source, _) =
                    fixture_from_runtime(execution, runtime(), false, 16);
                if tail_only {
                    let tail = source.commits.values().map(|bytes| {
                    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap()
                }).collect();
                    recovery = RelationalDurableRecoverySource::new(
                        None,
                        tail,
                        Vec::new(),
                        source.clone(),
                    );
                }
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
                runtime.history.reclaim_retired_branch_roots(4096);
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
                            .history()
                            .demand_commit_envelope(execution, receipt.commit_id),
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
                assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
            }
        }
    }
}
