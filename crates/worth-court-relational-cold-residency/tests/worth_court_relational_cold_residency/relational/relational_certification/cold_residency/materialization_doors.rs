//! Public materialization reads demand their actual recovered root footprint.
use super::*;
use worth_relational::facade::branch::{
    RelationalMaterializationError, RelationalMaterializationRecord,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_relational::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};

#[test]
fn residency_materialization_source_manifest_demands_before_reading_or_refusing_damage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
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
                worth_relational::facade::storage::RelationalStorageSubject::Region(
                    fixture.entity.partition_id
                )
            );
            assert_eq!(failure.cause, Cause::Damaged);
        } else {
            let (_, completion) = result
                .expect("the complete live cold source prepares its manifest")
                .into_parts();
            assert!(completion.source_records().iter().any(|record| matches!(record, RelationalMaterializationRecord::Entity { entity_id, .. } if *entity_id == fixture.entity)));
            assert!(completion.source_records().iter().any(|record| matches!(record, RelationalMaterializationRecord::Relation { relation_id, .. } if *relation_id == fixture.relation)));
        }
        assert!(fixture.source.reads.load(Ordering::Relaxed) > 0);
    }
}

#[test]
fn retained_kind_after_public_suspension_preserves_kind_and_storage_refusal() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let endpoint = create_entity(execution, &producer, "retained-endpoint");
        let mut unrelated_transaction = begin_main_transaction(&producer);
        unrelated_transaction
            .push_batch(
                WorkerIntentBatch::new("retained-other-partition").push(MutationIntent::Create(
                    CreateIntent::Entity(worth_relational::facade::transactions::EntitySpec {
                        partition_id: PartitionId(2),
                        kind_id: KindId(1),
                        client_key: worth_relational::facade::symbols::ClientKey::raw(
                            "retained-unrelated",
                        ),
                        fields: name_field_patch("retained-unrelated"),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let unrelated = unrelated_transaction.commit(execution, &producer).unwrap();
        let retained = changed_entities(&unrelated)[0];
        release_test_commit_snapshot(&producer, &unrelated);
        let created = create_group_with_external_endpoint(execution, &producer, endpoint);
        let internal = changed_entities(&created)[0];
        release_test_commit_snapshot(&producer, &created);
        let (runtime, source, _, _medium) =
            capture_and_reopen(execution, producer, medium, retained);
        let services = runtime.owner_component_services();
        let port = services.materialization_port();
        let (_, source_basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let (candidate, completion) = port
            .prepare_generated_materialization_suspension(execution, &source_basis, &[internal])
            .unwrap()
            .into_parts();
        let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) =
            services
                .publication_port()
                .compare_and_publish(execution, candidate)
        else {
            panic!("the actual prepared suspension publishes");
        };
        let committed = services
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        let suspended = completion.complete(committed).unwrap();
        release_test_commit_snapshot(&runtime, &suspended.commit);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
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
            }
            Err(RelationalMaterializationError::Read(
                RelationalExecutionDenialCause::StorageUnavailable(failure),
            )) => {
                assert!(damaged);
                assert_eq!(failure.cause, Cause::Damaged);
                assert_eq!(
                    failure.subject,
                    worth_relational::facade::storage::RelationalStorageSubject::Region(
                        retained.partition_id
                    )
                );
            }
            Err(other) => panic!("retained kind returned the wrong refusal: {other:?}"),
        }
    }
}

#[test]
fn residency_load_free_sharing_and_ledger_refuse_cold_payloads_without_inventing_absence() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    let identity = fixture.runtime.main_branch_identity();
    let before = fixture.source.reads.load(Ordering::Relaxed);
    assert!(matches!(
        fixture.runtime.observe_branch_sharing(execution, std::slice::from_ref(&identity)),
        Err(
            worth_relational::facade::inspection::RelationalBranchSharingInspectionDenial::StorageUnavailable(_)
        )
    ));
    assert!(matches!(
        fixture.runtime.inspect_owner_allocation_ledger(&[identity]),
        Err(
            worth_relational::facade::inspection::RelationalBranchSharingInspectionDenial::StorageUnavailable(_)
        )
    ));
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        before,
        "load-free inspection has no request with which to demand payloads"
    );
    assert!(matches!(fixture.runtime.observe_retained_storage(),
        Err(worth_relational::facade::inspection::RelationalRetainedStorageInspectionDenial::OwnerInventory(
            worth_relational::facade::inspection::RelationalBranchSharingInspectionDenial::StorageUnavailable(_)))));
    assert_eq!(
        fixture.source.reads.load(Ordering::Relaxed),
        before,
        "the all-retained census also refuses without demanding cold payloads"
    );
}
