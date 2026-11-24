use super::{current_snapshot, lifecycle_aspect};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use worth_relational::facade::commit_strategies::{
    ReplicaConvergenceAction, ReplicaConvergenceInput, ReplicaConvergenceOutput,
    ReplicaConvergenceStrategy,
};
use worth_relational::facade::commit_strategies::{
    StrategyCallerProvenance, StrategyRequestOrigin,
};
use worth_testkit_relational::aspect_storage::{
    entity_field_aspect, entity_u64_field_aspect, AspectSchemaFixture,
};
use worth_testkit_relational::ordinary_records::create_entity;

fn strategy_registry() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                worth_testkit_relational::aspect_storage::aspect_key("name"),
                worth_testkit_relational::aspect_storage::field_key("name"),
            ),
            entity_u64_field_aspect(
                worth_testkit_relational::aspect_storage::aspect_key("replicas"),
                worth_testkit_relational::aspect_storage::field_key("replicas"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry()
}

fn replicas_patch_target() -> AspectFieldLocator {
    worth_relational::facade::transactions::planned_single_field_locator(
        worth_testkit_relational::aspect_storage::aspect_key("replicas"),
        worth_testkit_relational::aspect_storage::field_key("replicas"),
    )
}

#[test]
fn replica_convergence_strategy_updates_replicas_and_preserves_other_fields() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = ReplicaConvergenceStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(601),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &ReplicaConvergenceInput {
                entity_id: entity,
                desired_replicas: 5,
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical request");
    let snapshot = current_snapshot(&runtime);
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy execution");
    let output = ReplicaConvergenceOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");
    let intent = &execution.mutation_program().worker_batches()[0].intents[0];
    let updated_replicas = match intent {
        worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(intent),
        ) => intent.fields.get(&replicas_patch_target()),
        other => panic!("expected update entity fields intent, got {other:?}"),
    };

    assert_eq!(output.action, ReplicaConvergenceAction::UpdateReplicas);
    assert_eq!(updated_replicas, Some(&AspectValue::UInt64(5)));
}

#[test]
fn replica_convergence_strategy_noops_when_authoritative_replicas_match() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = ReplicaConvergenceStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(602),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let mut txn =
        worth_testkit_relational::aspect_storage::publications::begin_main_transaction(&runtime);
    txn.push_batch(
        worth_relational::facade::transactions::WorkerIntentBatch::new("seed-replicas").push(
            worth_relational::facade::transactions::MutationIntent::Entity(
                worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                    worth_relational::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields:
                            worth_relational::facade::transactions::AspectFieldPatch::from_locator(
                                replicas_patch_target(),
                                AspectValue::UInt64(5),
                            ),
                    },
                ),
            ),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(request_execution, &runtime)
        .expect("seed replicas");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &ReplicaConvergenceInput {
                entity_id: entity,
                desired_replicas: 5,
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical request");
    let snapshot = current_snapshot(&runtime);
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy execution");
    let output = ReplicaConvergenceOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");

    assert_eq!(output.action, ReplicaConvergenceAction::NoChange);
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}
