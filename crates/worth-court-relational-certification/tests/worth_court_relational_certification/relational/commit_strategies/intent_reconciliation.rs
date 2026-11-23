use super::{current_snapshot, lifecycle_aspect};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::commit_strategies::{
    IntentReconciliationAction, IntentReconciliationInput, IntentReconciliationOutput,
    IntentReconciliationStrategy,
};
use worth_relational::facade::commit_strategies::{
    StrategyCallerProvenance, StrategyRequestOrigin,
};
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_testkit_relational::aspect_storage::{
    entity_field_aspect, entity_u64_field_aspect, AspectSchemaFixture,
};
use worth_testkit_relational::cold_recovery::update_entity;
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

fn name_patch_target() -> AspectFieldLocator {
    worth_relational::facade::transactions::planned_single_field_locator(
        worth_testkit_relational::aspect_storage::aspect_key("name"),
        worth_testkit_relational::aspect_storage::field_key("name"),
    )
}

#[test]
fn intent_reconciliation_strategy_emits_update_when_aspect_fields_differ() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = IntentReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(501),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &IntentReconciliationInput {
                entity_id: entity,
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    name_patch_target(),
                    AspectValue::String(InternedString::Raw("after".to_string())),
                ),
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
    let output = IntentReconciliationOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");

    assert_eq!(output.action, IntentReconciliationAction::UpdateEntity);
    assert_eq!(execution.mutation_program().total_intent_count(), 1);
}

#[test]
fn intent_reconciliation_strategy_emits_noop_when_aspect_fields_match() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = IntentReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(502),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    update_entity(request_execution, &runtime, entity, "stable");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &IntentReconciliationInput {
                entity_id: entity,
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    name_patch_target(),
                    AspectValue::String(InternedString::Raw("stable".to_string())),
                ),
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
    let output = IntentReconciliationOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");

    assert_eq!(output.action, IntentReconciliationAction::NoChange);
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}

#[test]
fn intent_reconciliation_strategy_preserves_untouched_declared_fields() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = IntentReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(503),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &IntentReconciliationInput {
                entity_id: entity,
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    replicas_patch_target(),
                    AspectValue::UInt64(3),
                ),
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
    let intent = &execution.mutation_program().worker_batches()[0].intents[0];
    let updated_aspect_value = match intent {
        worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(intent),
        ) => intent.fields.get(&replicas_patch_target()),
        other => panic!("expected update entity fields intent, got {other:?}"),
    };

    assert_eq!(updated_aspect_value, Some(&AspectValue::UInt64(3)));
}
