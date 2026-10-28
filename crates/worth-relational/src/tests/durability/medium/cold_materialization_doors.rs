//! Public materialization reads demand their actual recovered root footprint.
use super::*;
use crate::branch::{RelationalMaterializationError, RelationalMaterializationRecord};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::execution::RelationalExecutionDenialCause;
use crate::identity::data::{KindId, PartitionId};
use crate::storage::residency::{PartitionHeadProbe, PartitionProbe};
use crate::transactions::data::{CreateIntent, MutationIntent, WorkerIntentBatch};

#[test]
fn residency_materialization_source_manifest_demands_before_reading_or_refusing_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let fixture = CurrentFixture::reopened(execution);
        if damaged {
            for locator in fixture.source.members.keys() {
                fixture
                    .source
                    .failures
                    .lock()
                    .unwrap()
                    .insert(ColdRead::Member(*locator), Cause::Damaged);
            }
        }
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        let result = fixture
            .runtime
            .owner_component_services()
            .materialization_port()
            .prepare_generated_materialization_suspension(execution, &basis, &[fixture.entity]);
        if damaged {
            let Err(RelationalMaterializationError::Read(
                RelationalExecutionDenialCause::StorageUnavailable(failure),
            )) = result
            else {
                panic!("a damaged source manifest must preserve its native storage refusal");
            };
            assert_eq!(
                failure.subject,
                crate::storage::data::RelationalStorageSubject::Region(fixture.entity.partition_id)
            );
            assert_eq!(failure.cause, Cause::Damaged);
        } else {
            let (_, completion) = result
                .expect("the complete live cold source prepares its manifest")
                .into_parts();
            assert!(completion.records.iter().any(|record| matches!(record, RelationalMaterializationRecord::Entity { entity_id, .. } if *entity_id == fixture.entity)));
            assert!(completion.records.iter().any(|record| matches!(record, RelationalMaterializationRecord::Relation { relation_id, .. } if *relation_id == fixture.relation)));
        }
        assert!(fixture.source.reads.load(Ordering::Relaxed) > 0);
    }
}

#[test]
fn residency_retained_kind_demands_only_its_cold_head_and_preserves_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let producer = durable_runtime_with_test_schema();
        let endpoint = create_entity(execution, &producer, "retained-endpoint");
        let mut unrelated_transaction = test_owner_begin_transaction_for_main(&producer);
        unrelated_transaction
            .push_batch(
                WorkerIntentBatch::new("retained-other-partition").push(MutationIntent::Create(
                    CreateIntent::Entity(crate::transactions::data::EntitySpec {
                        partition_id: PartitionId(2),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("retained-unrelated"),
                        fields: crate::tests::support::name_field_patch("retained-unrelated"),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let unrelated = unrelated_transaction.commit(execution, &producer).unwrap();
        let retained = changed_entities(&unrelated)[0];
        release_test_commit_snapshot(&producer, &unrelated);
        let created = crate::tests::support::create_group_with_external_endpoint(
            execution, &producer, endpoint,
        );
        let internal = changed_entities(&created)[0];
        release_test_commit_snapshot(&producer, &created);
        let (recovery, source, _) = checkpoint_fixture(execution, producer, retained, false);
        source.damaged.store(false, Ordering::Relaxed);
        let attachment = TestDurableMedium::new().attachment();
        for bytes in source.commits.values() {
            let artifact =
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
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
        let services = runtime.owner_component_services();
        let port = services.materialization_port();
        let (_, source_basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let (candidate, completion) = port
            .prepare_generated_materialization_suspension(execution, &source_basis, &[internal])
            .unwrap()
            .into_parts();
        let committed = runtime
            .publish_prepared_candidate(execution, candidate)
            .unwrap();
        let suspended = completion.complete(committed).unwrap();
        release_test_commit_snapshot(&runtime, &suspended.commit);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        assert!(
            matches!(
                basis.inner.root.probe_head_region(retained.partition_id),
                PartitionHeadProbe::Cold
            ),
            "the unrelated retained identity must still be cold at this public door"
        );
        if damaged {
            for locator in source.members.keys() {
                source
                    .failures
                    .lock()
                    .unwrap()
                    .insert(ColdRead::Member(*locator), Cause::Damaged);
            }
        }
        match port.retained_entity_kind(execution, &basis, &suspended.custody, retained) {
            Ok(kind) => {
                assert!(!damaged);
                assert_eq!(kind, KindId(1));
                assert!(
                    matches!(
                        basis.inner.root.probe_full_region(retained.partition_id),
                        PartitionProbe::Cold
                    ),
                    "a value-free kind read must not demand Full history"
                );
            }
            Err(RelationalMaterializationError::Read(
                RelationalExecutionDenialCause::StorageUnavailable(failure),
            )) => {
                assert!(damaged);
                assert_eq!(failure.cause, Cause::Damaged);
                assert_eq!(
                    failure.subject,
                    crate::storage::data::RelationalStorageSubject::Region(retained.partition_id)
                );
            }
            Err(other) => panic!("retained kind returned the wrong refusal: {other:?}"),
        }
    }
}

#[test]
fn residency_load_free_sharing_and_ledger_refuse_cold_payloads_without_inventing_absence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let identity = fixture.runtime.main_branch_identity();
    let before = fixture.source.reads.load(Ordering::Relaxed);
    assert!(matches!(
        fixture.runtime.observe_branch_sharing(std::slice::from_ref(&identity)),
        Err(
            crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial::StorageUnavailable(_)
        )
    ));
    assert!(matches!(
        fixture.runtime.inspect_owner_allocation_ledger(&[identity]),
        Err(
            crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial::StorageUnavailable(_)
        )
    ));
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        before,
        "load-free inspection has no request with which to demand payloads"
    );
}
