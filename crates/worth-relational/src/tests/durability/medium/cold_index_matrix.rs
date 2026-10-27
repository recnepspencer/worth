//! Restored index payloads must be admitted before pinning or preparation.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::indexes::data::DerivedIndexId;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[derive(Debug, Clone, Copy)]
enum IndexDoor {
    Build,
    BasisBuild,
    MemoryCheckpoint,
    EmbeddedCheckpoint,
    SnapshotPin,
    ViewPin,
    FieldPreparation,
    Validation,
    Candidate,
    Commit,
}
impl IndexDoor {
    const ALL: [Self; 10] = [
        Self::Build,
        Self::BasisBuild,
        Self::MemoryCheckpoint,
        Self::EmbeddedCheckpoint,
        Self::SnapshotPin,
        Self::ViewPin,
        Self::FieldPreparation,
        Self::Validation,
        Self::Candidate,
        Self::Commit,
    ];
    fn read(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: DerivedIndexId,
        expected_generation: &crate::indexes::data::DerivedIndexGeneration,
    ) -> Result<bool, Execution> {
        use crate::runtime::RelationalPinnedReadAdmissionDenial as Pin;
        let pin_denial = |denial| match denial {
            Pin::Request(stop) => stop,
            other => {
                panic!("the actual owner basis and installed definition remain valid: {other:?}")
            }
        };
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        match self {
            Self::MemoryCheckpoint | Self::EmbeddedCheckpoint => {
                super::cold_index_checkpoint_door::read(
                    execution,
                    runtime,
                    matches!(self, Self::EmbeddedCheckpoint),
                    expected_generation,
                )
            }
            Self::Build | Self::BasisBuild => {
                let door = if matches!(self, Self::Build) {
                    super::cold_index_build_door::IndexBuildDoor::Commit
                } else {
                    super::cold_index_build_door::IndexBuildDoor::Basis
                };
                door.read(runtime, execution, index).map(|_| {
                    let successor = runtime
                        .index_access()
                        .latest_generation(index, &expected_generation.source_branch_id)
                        .unwrap();
                    assert!(successor.generation_id > expected_generation.generation_id);
                    assert!(
                        successor.generation_id.0 < expected_generation.generation_id.0 + 16,
                        "successor generation growth has headroom"
                    );
                    assert_eq!(successor.entries, expected_generation.entries);
                    true
                })
            }
            Self::SnapshotPin => {
                let handle = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let answer = runtime
                    .read_truth()
                    .pin_read_capability(&handle, &[index], execution)
                    .map(|pin| pin.index_generation(index).is_some())
                    .map_err(pin_denial);
                runtime.snapshots().release_snapshot(&handle).unwrap();
                answer
            }
            Self::ViewPin | Self::FieldPreparation => {
                let view = runtime
                    .read_truth()
                    .project_observation(&basis.observation())
                    .unwrap();
                assert!(view.is_exact_basis());
                match self {
                    Self::ViewPin => runtime
                        .read_truth()
                        .pin_view_read_capability(&view, &[index], execution)
                        .map(|pin| pin.index_generation(index).is_some())
                        .map_err(pin_denial),
                    Self::FieldPreparation => runtime
                        .index_access()
                        .prepare_entity_field_lookup(
                            &view,
                            index,
                            KindId(1),
                            &aspect_field_locator(aspect_key("name"), field_key("name")),
                            execution,
                        )
                        .map(|_| true)
                        .map_err(|denial| match denial.kind() {
                            crate::indexes::data::BoundedEntityFieldLookupDenialKind::Execution(
                                cause,
                            ) => cause,
                            other => panic!("the actual field definition is valid: {other:?}"),
                        }),
                    _ => unreachable!("selected view index door"),
                }
            }
            Self::Validation | Self::Candidate | Self::Commit => {
                let mut transaction = test_owner_begin_transaction_for_main(runtime);
                transaction
                    .push_batch(
                        batch_create("index-matrix-write"),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                let result = match self {
                    Self::Validation => transaction.validate(execution, runtime).map(|_| true),
                    Self::Candidate => runtime
                        .prepare_branch_transaction(execution, transaction)
                        .map(|_| true),
                    Self::Commit => transaction.commit(execution, runtime).map(|outcome| {
                        release_test_commit_snapshot(runtime, &outcome);
                        true
                    }),
                    _ => unreachable!("selected preparation index door"),
                };
                result.map_err(|denial| {
                    let crate::transactions::data::TransactionCommitError::Preparation {
                        error,
                        ..
                    } = denial
                    else {
                        panic!("index storage refusal is pre-effect: {denial:?}");
                    };
                    let crate::transactions::data::CommitPreparationReason::StorageUnavailable(
                        failure,
                    ) = error.reason()
                    else {
                        panic!("index storage failure remains typed: {error:?}");
                    };
                    Execution::StorageUnavailable(failure)
                })
            }
        }
    }
}
#[test]
fn residency_failure_matrix_index_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in IndexDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = super::cold_index_fixture::reopened(execution);
            let runtime = fixture.runtime;
            let source = fixture.source;
            let entity = fixture.entity;
            let index = fixture.index;
            let generation = fixture.generation;
            let member = fixture.member;
            assert!(runtime
                .read_truth()
                .visible_entity_at_version(execution, entity, runtime.current_version_id())
                .unwrap()
                .is_some());
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&runtime).offered().len();
            source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(index),
                cause,
            });
            assert_eq!(
                door.read(&runtime, execution, index, &generation),
                Err(expected),
                "{door:?} x {cause:?}: a cold index must not become absence"
            );
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
            source.failures.lock().unwrap().clear();
            let reads = source.reads.load(Ordering::Relaxed);
            let failed_reads = || {
                source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|read| **read == member)
                    .count()
            };
            let failed_before = failed_reads();
            if sticky {
                assert_eq!(
                    door.read(&runtime, execution, index, &generation),
                    Err(expected)
                );
                if matches!(
                    door,
                    IndexDoor::MemoryCheckpoint | IndexDoor::EmbeddedCheckpoint
                ) {
                    assert_eq!(failed_reads(), failed_before);
                } else {
                    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                }
            } else {
                assert!(
                    door.read(&runtime, execution, index, &generation).unwrap(),
                    "retry must retain a genuine generation"
                );
                assert!(source.reads.load(Ordering::Relaxed) > reads);
            }
        }
    }
}
