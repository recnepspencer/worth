//! Restored index payloads must be admitted before pinning or preparation.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[derive(Debug, Clone, Copy)]
enum IndexDoor {
    SnapshotPin,
    ViewPin,
    FieldPreparation,
    Validation,
    Candidate,
    Commit,
}
impl IndexDoor {
    const ALL: [Self; 6] = [
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
            let producer = durable_runtime_with_declared_aspect_schema(
                CascadeDeletePolicy::CascadeDeleteRelations,
            );
            let index = producer
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: "matrix-name".into(),
                    branch_scoped: true,
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                    },
                })
                .index_id;
            let entity = create_entity(execution, &producer, "matrix-index-value");
            let branch = crate::history::data::BranchId("main".into());
            let head = producer.history().branch_head(&branch).unwrap();
            let built = producer.index_authority().build_for_commit(
                execution,
                crate::indexes::data::DerivedIndexBuildRequest {
                    source_commit_id: head.commit_id,
                    branch_id: branch.clone(),
                    index_ids: vec![index],
                },
            );
            assert!(
                built.failed_indexes.is_empty(),
                "the actual producer must build this index"
            );
            let generation = producer
                .index_access()
                .latest_generation(index, &branch)
                .unwrap();
            assert_eq!(generation.source_commit_id, head.commit_id);
            let (recovery, source, entity) = checkpoint_fixture(execution, producer, entity, false);
            source.damaged.store(false, Ordering::Relaxed);
            #[derive(serde::Deserialize)]
            struct Input {
                manifest: crate::durability::data::RelationalCheckpointManifest,
            }
            let manifest = rmp_serde::from_slice::<Input>(
                recovery
                    .checkpoint()
                    .unwrap()
                    .checkpoint()
                    .bytes()
                    .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
                    .unwrap(),
            )
            .unwrap()
            .manifest;
            assert!(manifest
                .index_definitions
                .iter()
                .any(|definition| definition.index_id == index));
            let attachment = TestDurableMedium::new().attachment();
            for bytes in source.commits.values() {
                let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
                assert!(matches!(
                    attachment.publish(&artifact),
                    crate::durability::medium::RelationalDurablePublication::Committed
                ));
            }
            let mut runtime = durable_runtime_with_declared_aspect_schema(
                CascadeDeletePolicy::CascadeDeleteRelations,
            );
            bind_durable_medium(&mut runtime, attachment);
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
            assert_eq!(source.reads.load(Ordering::Relaxed), 0);
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
                .insert(ColdRead::Member(manifest.derived_indexes), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(index),
                cause,
            });
            assert_eq!(
                door.read(&runtime, execution, index),
                Err(expected),
                "{door:?} x {cause:?}: a cold index must not become absence"
            );
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
            source.failures.lock().unwrap().clear();
            let reads = source.reads.load(Ordering::Relaxed);
            if sticky {
                assert_eq!(door.read(&runtime, execution, index), Err(expected));
                assert_eq!(source.reads.load(Ordering::Relaxed), reads);
            } else {
                assert!(
                    door.read(&runtime, execution, index).unwrap(),
                    "retry must retain a genuine generation"
                );
                assert!(source.reads.load(Ordering::Relaxed) > reads);
            }
        }
    }
}
