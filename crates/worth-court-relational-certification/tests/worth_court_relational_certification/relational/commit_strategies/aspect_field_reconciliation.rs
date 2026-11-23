use super::{current_snapshot, lifecycle_aspect};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::commit_strategies::{
    AspectFieldReconciliationInput, AspectFieldReconciliationOutput,
    AspectFieldReconciliationStrategy,
};
use worth_relational::facade::commit_strategies::{
    StrategyCallerProvenance, StrategyExecutorFailureClass, StrategyRequestOrigin,
};
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_testkit_relational::aspect_storage::{
    entity_field_aspect, entity_u64_field_aspect, AspectSchemaFixture,
};
use worth_testkit_relational::cold_recovery::update_entity;
use worth_testkit_relational::ordinary_records::create_entity;

fn strategy_runtime(
    descriptor: worth_relational::facade::commit_strategies::CommitStrategyDescriptor,
    registry: worth_relational::facade::schema::RelationalSchemaRegistry,
) -> worth_relational::facade::runtime::RelationalRuntime {
    worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(AspectFieldReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
        .expect("valid native runtime configuration")
}

fn registry_with_replicas_field() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    AspectSchemaFixture {
        cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
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

fn field_locator(aspect_key: AspectKey, field_key: FieldKey) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key,
        CanonicalFieldPath::single(field_key),
    )
}

fn replicas_patch_target() -> AspectFieldLocator {
    worth_relational::facade::transactions::planned_single_field_locator(
        worth_testkit_relational::aspect_storage::aspect_key("replicas"),
        worth_testkit_relational::aspect_storage::field_key("replicas"),
    )
}

#[test]
fn aspect_field_reconciliation_strategy_updates_only_declared_field_aspect() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = AspectFieldReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(701),
    );
    let runtime = strategy_runtime(descriptor, registry_with_replicas_field());
    let entity = create_entity(request_execution, &runtime, "before");
    update_entity(request_execution, &runtime, entity, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &AspectFieldReconciliationInput {
                entity_id: entity,
                field_locator: field_locator(
                    worth_testkit_relational::aspect_storage::aspect_key("replicas"),
                    worth_testkit_relational::aspect_storage::field_key("replicas"),
                ),
                desired_value: worth_foundational::facade::AspectValue::UInt64(5),
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
    let output = AspectFieldReconciliationOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");
    let updated_replicas = match &execution.mutation_program().worker_batches()[0].intents[0] {
        worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(intent),
        ) => intent.fields.get(&replicas_patch_target()),
        other => panic!("expected update entity fields intent, got {other:?}"),
    };

    assert!(output.updated);
    assert_eq!(
        output.field_locator,
        field_locator(
            worth_testkit_relational::aspect_storage::aspect_key("replicas"),
            worth_testkit_relational::aspect_storage::field_key("replicas")
        )
    );
    assert_eq!(
        updated_replicas,
        Some(&worth_foundational::facade::AspectValue::UInt64(5))
    );
}

#[test]
fn aspect_field_reconciliation_strategy_noops_when_authoritative_field_matches() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = AspectFieldReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(703),
    );
    let runtime = strategy_runtime(descriptor, registry_with_replicas_field());
    let entity = create_entity(request_execution, &runtime, "stable");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &AspectFieldReconciliationInput {
                entity_id: entity,
                field_locator: field_locator(
                    worth_testkit_relational::aspect_storage::aspect_key("name"),
                    worth_testkit_relational::aspect_storage::field_key("name"),
                ),
                desired_value: worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw("stable".to_string()),
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
    let output = AspectFieldReconciliationOutput::decode(execution.output().canonical_bytes())
        .expect("output decode");

    assert!(!output.updated);
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}

#[test]
fn aspect_field_reconciliation_strategy_rejects_undeclared_field() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = AspectFieldReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(702),
    );
    let registry = AspectSchemaFixture {
        cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
        entity_aspects: vec![
            entity_field_aspect(
                worth_testkit_relational::aspect_storage::aspect_key("name"),
                worth_testkit_relational::aspect_storage::field_key("name"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = strategy_runtime(descriptor, registry);
    let entity = create_entity(execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &AspectFieldReconciliationInput {
                entity_id: entity,
                field_locator: field_locator(
                    worth_testkit_relational::aspect_storage::aspect_key("replicas"),
                    worth_testkit_relational::aspect_storage::field_key("replicas"),
                ),
                desired_value: worth_foundational::facade::AspectValue::UInt64(5),
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
    let error = runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
        .expect_err("undeclared aspect field should be rejected");

    match error {
        worth_relational::facade::commit_strategies::StrategyExecutionError::ExecutorFailed {
            failure,
            ..
        } => {
            let worth_relational::facade::commit_strategies::StrategyExecutorFailure::Domain {
                class,
                detail,
                evidence,
                ..
            } = failure
            else {
                panic!("unexpected request stop: {failure:?}")
            };
            assert_eq!(class, StrategyExecutorFailureClass::DomainRejection);
            assert!(detail.contains(
                "aspect field locator is not a lowered foundational scalar entity aspect"
            ));
            let evidence = serde_json::to_value(evidence.unwrap()).unwrap();
            let encoded_locator: Vec<u8> =
                serde_json::from_value(evidence["AspectFieldLocator"]["locator"].clone()).unwrap();
            assert_eq!(
                encoded_locator
                    .windows(b"replicas".len())
                    .filter(|word| *word == b"replicas")
                    .count(),
                2,
                "the evidence names the refused aspect and its field",
            );
        }
        other => panic!("expected executor failure, got {other:?}"),
    }
}
