//! Current operations must not depend on unavailable historical payloads.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::{EntityMutationIntent, UpdateEntityFieldsIntent};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
enum CurrentDoor {
    Entity,
    Relation,
    Validation,
    Candidate,
    Commit,
    DeleteRelation,
}
impl CurrentDoor {
    const ALL: [Self; 6] = [
        Self::Entity,
        Self::Relation,
        Self::Validation,
        Self::Candidate,
        Self::Commit,
        Self::DeleteRelation,
    ];
    fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: EntityId,
        relation: RelationId,
    ) {
        let mut transaction = test_owner_begin_transaction_for_main(runtime);
        match self {
            Self::DeleteRelation => {
                let outcome = crate::tests::support::delete_relation_on_branch(
                    execution,
                    runtime,
                    relation,
                    BranchId("main".to_owned()),
                );
                release_test_commit_snapshot(runtime, &outcome);
                assert_eq!(
                    transaction
                        .read_relation(execution, relation)
                        .unwrap()
                        .effective()
                        .unwrap()
                        .retired_at_version,
                    None,
                    "the transaction retained the pre-delete observation"
                );
                let mut current = test_owner_begin_transaction_for_main(runtime);
                assert_eq!(
                    current
                        .read_relation(execution, relation)
                        .unwrap()
                        .effective()
                        .unwrap()
                        .retired_at_version,
                    Some(runtime.current_version_id())
                );
            }
            Self::Entity => assert_eq!(
                transaction
                    .read_entity(execution, entity)
                    .unwrap()
                    .effective()
                    .unwrap()
                    .entity_id,
                entity
            ),
            Self::Relation => assert_eq!(
                transaction
                    .read_relation(execution, relation)
                    .unwrap()
                    .effective()
                    .unwrap()
                    .relation_id,
                relation
            ),
            Self::Validation | Self::Candidate | Self::Commit => {
                transaction
                    .push_batch(
                        WorkerIntentBatch::new("head-only-current-write").push(
                            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                                UpdateEntityFieldsIntent {
                                    entity_id: entity,
                                    fields: name_field_patch("head-only-after"),
                                },
                            )),
                        ),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                match self {
                    Self::Validation => {
                        transaction.validate(execution, runtime).unwrap();
                    }
                    Self::Candidate => {
                        runtime
                            .prepare_branch_transaction(execution, transaction)
                            .unwrap();
                    }
                    Self::Commit => {
                        let outcome = transaction.commit(execution, runtime).unwrap();
                        release_test_commit_snapshot(runtime, &outcome);
                    }
                    _ => unreachable!("selected current write door"),
                }
            }
        }
    }
}

struct CurrentFixture {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    entity: EntityId,
    relation: RelationId,
    history: BTreeSet<RelationalDurableMemberLocator>,
}
impl CurrentFixture {
    fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let producer = durable_runtime_with_test_schema();
        let entity = create_entity(execution, &producer, "head-only-before");
        let target = create_entity(execution, &producer, "head-only-target");
        let linked =
            create_relation_outcome(execution, &producer, entity, target, "head-only-edge");
        let relation = changed_relations(&linked)[0];
        release_test_commit_snapshot(&producer, &linked);
        update_entity_and_release_snapshot(execution, &producer, entity, "head-only-current");
        let (recovery, source, _) = checkpoint_fixture(execution, producer, entity, false);
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
        let history = manifest
            .mirror
            .iter()
            .chain(manifest.roots.iter().flat_map(|root| root.regions.iter()))
            .filter(|region| region.partition_id == entity.partition_id)
            .map(|region| region.history)
            .collect();
        let attachment = TestDurableMedium::new().attachment();
        for bytes in source.commits.values() {
            let artifact = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
            assert!(matches!(
                attachment.publish(&artifact),
                crate::durability::medium::RelationalDurablePublication::Committed
            ));
        }
        let mut runtime = durable_runtime_with_test_schema();
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
        Self {
            runtime,
            source,
            entity,
            relation,
            history,
        }
    }
}

#[test]
fn residency_failure_matrix_current_doors_never_consult_region_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in CurrentDoor::ALL {
        for cause in FAILURES {
            let fixture = CurrentFixture::reopened(execution);
            assert!(!fixture.history.is_empty());
            fixture.source.failures.lock().unwrap().extend(
                fixture
                    .history
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            door.run(
                execution,
                &fixture.runtime,
                fixture.entity,
                fixture.relation,
            );
            assert!(
                !fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|member| fixture.history.contains(member)),
                "{door:?} x {cause:?} opened history"
            );
            if matches!(door, CurrentDoor::Commit | CurrentDoor::DeleteRelation) {
                assert!(fixture.runtime.history().latest_patch_stream_position() > frontier);
                assert!(attached_test_medium(&fixture.runtime).offered().len() > offered);
                let mut transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
                assert_eq!(
                    transaction
                        .read_entity(execution, fixture.entity)
                        .unwrap()
                        .effective()
                        .unwrap()
                        .entity_id,
                    fixture.entity
                );
            } else {
                assert_eq!(
                    fixture.runtime.history().latest_patch_stream_position(),
                    frontier
                );
                assert_eq!(
                    attached_test_medium(&fixture.runtime).offered().len(),
                    offered
                );
            }
        }
    }
}

#[test]
fn head_write_capture_and_full_promotion_preserve_current_and_retained_reference_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let (_, before) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let observation = before.observation();
    let reference = runtime_with_test_schema();
    let expected_entity = create_entity(execution, &reference, "head-only-before");
    let expected_target = create_entity(execution, &reference, "head-only-target");
    let linked = create_relation_outcome(
        execution,
        &reference,
        expected_entity,
        expected_target,
        "head-only-edge",
    );
    let expected_relation = changed_relations(&linked)[0];
    release_test_commit_snapshot(&reference, &linked);
    update_entity_and_release_snapshot(execution, &reference, expected_entity, "head-only-current");
    let (_, expected_before) = reference
        .observe_branch(&reference.main_branch_identity())
        .unwrap();
    let expected_old = reference
        .read_truth()
        .read_observation(execution, &expected_before.observation())
        .unwrap()
        .unwrap();
    fixture.source.failures.lock().unwrap().extend(
        fixture
            .history
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::SourceUnavailable)),
    );
    CurrentDoor::Commit.run(
        execution,
        &fixture.runtime,
        fixture.entity,
        fixture.relation,
    );
    CurrentDoor::Commit.run(execution, &reference, expected_entity, expected_relation);
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    let members = fixture.source.members.keys().copied().collect::<Vec<_>>();
    let checkpoint = fixture
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members,
                source: Some(fixture.source.clone()),
            },
        )
        .unwrap();
    assert!(!checkpoint.members().is_empty());
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        reads,
        "Head capture cannot load cold contents"
    );
    let frontier = fixture.runtime.history().latest_patch_stream_position();
    let failure = fixture
        .runtime
        .read_truth()
        .visible_entity_at_version(
            execution,
            fixture.entity,
            fixture.runtime.current_version_id(),
        )
        .unwrap_err();
    assert!(
        matches!(failure, crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
        crate::storage::data::RelationalStorageUnavailable {subject: crate::storage::data::RelationalStorageSubject::Region(partition), cause: Cause::SourceUnavailable}) if partition == fixture.entity.partition_id)
    );
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        frontier
    );
    fixture.source.failures.lock().unwrap().clear();
    let full = fixture
        .runtime
        .read_truth()
        .visible_entity_at_version(
            execution,
            fixture.entity,
            fixture.runtime.current_version_id(),
        )
        .unwrap()
        .unwrap();
    let expected = reference
        .read_truth()
        .visible_entity_at_version(execution, expected_entity, reference.current_version_id())
        .unwrap()
        .unwrap();
    assert_eq!(full, expected);
    let retained = fixture
        .runtime
        .read_truth()
        .read_observation(execution, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities(), expected_old.entities());
    assert_eq!(retained.relations(), expected_old.relations());
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    assert_eq!(
        fixture
            .runtime
            .read_truth()
            .visible_entity_at_version(
                execution,
                fixture.entity,
                fixture.runtime.current_version_id()
            )
            .unwrap()
            .unwrap(),
        expected
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
}
