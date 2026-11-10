//! Reserved candidate publication survives actual selected member replacement and ACK.
use super::{
    current_field_meanings::name_field_patch, current_publications::create_entity,
    merge_publications, selected_directory,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, identity::*, runtime::RelationalRuntime, transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
fn publish_checkpoint(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &std::sync::Arc<TestDurableMedium>,
    current: &[RelationalDurableMemberLocator],
) -> RelationalNativeCheckpoint {
    let source: std::sync::Arc<dyn RelationalDurableColdSource> = medium.clone();
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: current,
                source: Some(source),
            },
        )
        .unwrap();
    medium.file_checkpoint(native.clone());
    request
        .run_native_with_checkpoint(|context| native.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();
    native
}
#[test]
fn a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "reservation-anchor");
    let other = merge_publications::branch(execution, &runtime, "reservation-other");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("reserved-lower-page-id").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "reserved-lower-page-id".into(),
                    fields: name_field_patch("reserved-lower-page-id"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    merge_publications::create(
        execution,
        &runtime,
        "published-higher-page-id",
        PartitionId(2),
        &other,
    );
    let first = publish_checkpoint(execution, &runtime, &medium, &[]);
    let current = first
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect::<Vec<_>>();
    use worth_relational::facade::mvcc::RelationalPublicationOutcome;
    let performed = match runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
        RelationalPublicationOutcome::RequestStopped(cause) => {
            panic!("candidate request stopped: {cause:?}")
        }
        RelationalPublicationOutcome::Stale(cause) => panic!("candidate branch stale: {cause:?}"),
        RelationalPublicationOutcome::Denied(cause) => panic!("candidate denied: {cause:?}"),
        RelationalPublicationOutcome::Interrupted(cause) => {
            panic!("candidate interrupted: {cause:?}")
        }
        RelationalPublicationOutcome::Deferred(cause) => panic!("candidate deferred: {cause:?}"),
        RelationalPublicationOutcome::Failed(cause) => panic!("candidate failed: {cause:?}"),
        RelationalPublicationOutcome::DurabilityUnresolved(cause) => {
            panic!("candidate durability unresolved: {cause:?}")
        }
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(&runtime, &committed);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let expected_node = runtime
        .lineage_node_at_observation(execution, &basis.observation(), entity)
        .unwrap()
        .unwrap()
        .node()
        .clone();
    drop(basis);
    let second = publish_checkpoint(execution, &runtime, &medium, &current);
    let source = medium.recovery_source();
    assert_eq!(
        source.checkpoint().unwrap().checkpoint().bytes(),
        second.bytes()
    );
    let directory = selected_directory::read(&source);
    let mut reopened = durable_runtime(&medium);
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let mut reader = begin_main_transaction(&reopened);
    assert!(
        reader
            .read_entity(execution, entity)
            .unwrap()
            .effective()
            .is_some(),
        "published record survives checkpoint"
    );
    let (_, basis) = reopened
        .observe_branch(&reopened.main_branch_identity())
        .unwrap();
    assert_eq!(
        reopened
            .lineage_node_at_observation(execution, &basis.observation(), entity)
            .unwrap()
            .map(|node| node.node().clone()),
        Some(expected_node)
    );
    assert!(reopened
        .entity_record_at_observation(execution, &basis.observation(), entity)
        .unwrap()
        .is_some());
    assert!(!directory.lineage.is_empty());
}
