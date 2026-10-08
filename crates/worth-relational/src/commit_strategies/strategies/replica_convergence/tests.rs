use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use super::{
    ReplicaConvergenceAction, ReplicaConvergenceInput, ReplicaConvergenceOutput,
    ReplicaConvergenceStrategy,
};
use crate::commit_strategies::data::{StrategyCallerProvenance, StrategyRequestOrigin};
use crate::runtime::builder::RelationalRuntimeBuilder;
use crate::tests::support::{
    entity_field_aspect, entity_u64_field_aspect, lifecycle_aspect, AspectSchemaFixture,
};

fn strategy_registry() -> crate::schema::data::RelationalSchemaRegistry {
    AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_u64_field_aspect(
                crate::tests::support::aspect_key("replicas"),
                crate::tests::support::field_key("replicas"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry()
}

fn replicas_patch_target() -> AspectFieldLocator {
    crate::transactions::data::planned_single_field_locator(
        crate::tests::support::aspect_key("replicas"),
        crate::tests::support::field_key("replicas"),
    )
}

#[test]
fn replica_convergence_strategy_updates_replicas_and_preserves_other_fields() {
    let descriptor = ReplicaConvergenceStrategy::descriptor(
        crate::commit_strategies::data::CommitStrategyId(601),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            crate::commit_strategies::data::CommitStrategyRegistration::new(descriptor.clone())
                .expect("strategy registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &descriptor,
        ))
        .build();
    let entity = crate::tests::support::create_entity(&runtime, "before");
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");
    let output = ReplicaConvergenceOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");
    let intent = &execution.mutation_program().worker_batches()[0].intents[0];
    let updated_replicas = match intent {
        crate::transactions::data::MutationIntent::Entity(
            crate::transactions::data::EntityMutationIntent::UpdateFields(intent),
        ) => intent.fields.get(&replicas_patch_target()),
        other => panic!("expected update entity fields intent, got {other:?}"),
    };

    assert_eq!(output.action, ReplicaConvergenceAction::UpdateReplicas);
    assert_eq!(updated_replicas, Some(&AspectValue::UInt64(5)));
}

#[test]
fn replica_convergence_strategy_noops_when_authoritative_replicas_match() {
    let descriptor = ReplicaConvergenceStrategy::descriptor(
        crate::commit_strategies::data::CommitStrategyId(602),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            crate::commit_strategies::data::CommitStrategyRegistration::new(descriptor.clone())
                .expect("strategy registration"),
        )
        .commit_strategy_executor(ReplicaConvergenceStrategy::execution_registration(
            &descriptor,
        ))
        .build();
    let entity = crate::tests::support::create_entity(&runtime, "before");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        crate::transactions::data::WorkerIntentBatch::new("seed-replicas").push(
            crate::transactions::data::MutationIntent::Entity(
                crate::transactions::data::EntityMutationIntent::UpdateFields(
                    crate::transactions::data::UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: crate::transactions::data::AspectFieldPatch::from_locator(
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
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");
    let output = ReplicaConvergenceOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");

    assert_eq!(output.action, ReplicaConvergenceAction::NoChange);
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}
