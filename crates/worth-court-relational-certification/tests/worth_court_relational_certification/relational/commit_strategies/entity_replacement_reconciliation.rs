use super::{current_snapshot, lifecycle_aspect};
use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectValue, InternedString};
use worth_relational::facade::commit_strategies::{
    EntityReplacementReconciliationAction, EntityReplacementReconciliationInput,
    EntityReplacementReconciliationOutput, EntityReplacementReconciliationStrategy,
};
use worth_relational::facade::commit_strategies::{
    StrategyCallerProvenance, StrategyExecutorFailureClass, StrategyRequestOrigin,
};
use worth_relational::facade::transactions::AspectFieldPatch;
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

fn name_patch_target() -> AspectFieldLocator {
    worth_relational::facade::transactions::planned_single_field_locator(
        worth_testkit_relational::aspect_storage::aspect_key("name"),
        worth_testkit_relational::aspect_storage::field_key("name"),
    )
}

#[test]
fn entity_replacement_reconciliation_strategy_replaces_with_normalized_client_key_and_preserved_fields(
) {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(711),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: "service-replacement".to_string(),
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
    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("output decode");
    let replacement = match &execution.mutation_program().worker_batches()[0].intents[0] {
        worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::Replace(intent),
        ) => intent,
        other => panic!("expected replace entity intent, got {other:?}"),
    };
    let replacement_replicas = replacement.replacement.fields.get(&replicas_patch_target());

    assert_eq!(
        output.action,
        EntityReplacementReconciliationAction::ReplaceEntity
    );
    assert_eq!(
        replacement.replacement.client_key,
        worth_relational::facade::symbols::ClientKey::raw("service-replacement")
    );
    assert_eq!(replacement_replicas, Some(&AspectValue::UInt64(3)));
}

#[test]
fn entity_replacement_reconciliation_strategy_replacement_declaration_applies_to_authority() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(713),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: "service-replacement".to_string(),
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    replicas_patch_target(),
                    AspectValue::UInt64(7),
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
    let mut txn =
        worth_testkit_relational::aspect_storage::publications::begin_main_transaction(&runtime);
    for batch in execution
        .mutation_program()
        .worker_batches()
        .iter()
        .cloned()
    {
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    }
    let commit = txn
        .commit(request_execution, &runtime)
        .expect("replacement patch commit");
    let replacement_id =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&commit)
            .into_iter()
            .last()
            .expect("replacement entity id");
    let replacement_snapshot = current_snapshot(&runtime);
    let replacement_read = runtime
        .read_truth()
        .read_snapshot(&replacement_snapshot, request_execution)
        .expect("read request remains live")
        .expect("replacement snapshot should read");
    let replacement_record = replacement_read
        .get_entity(replacement_id)
        .expect("replacement record should be visible");

    let replicas = replacement_record
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(&AspectKey::new("replicas").unwrap())
        .unwrap();
    assert!(matches!(
        replicas.view(),
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(AspectValue::UInt64(
            7
        ))
    ));
}

#[test]
fn entity_replacement_reconciliation_strategy_rejects_undeclared_fields() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(712),
    );
    let registry = AspectSchemaFixture {
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
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .commit_strategy(
            worth_relational::facade::commit_strategies::CommitStrategyRegistration::new(
                descriptor.clone(),
            )
            .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(execution, &runtime, "before");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: "service-replacement".to_string(),
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
    let error = runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
        .expect_err("undeclared replacement fields should fail");

    match error {
        worth_relational::facade::commit_strategies::StrategyExecutionError::ExecutorFailed {
            failure,
            ..
        } => {
            let worth_relational::facade::commit_strategies::StrategyExecutorFailure::Domain {
                class,
                detail,
                ..
            } = failure
            else {
                panic!("unexpected request stop: {failure:?}")
            };
            assert_eq!(class, StrategyExecutorFailureClass::DomainRejection);
            assert!(detail.contains("not a lowered foundational scalar entity aspect"));
        }
        other => panic!("expected executor failure, got {other:?}"),
    }
}

#[test]
fn entity_replacement_reconciliation_strategy_replaces_when_only_client_key_changes() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        worth_relational::facade::commit_strategies::CommitStrategyId(77),
    );
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(strategy_registry())
        .commit_strategy(
            EntityReplacementReconciliationStrategy::registration(descriptor.id())
                .expect("registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build()
        .expect("valid native runtime configuration");
    let entity = create_entity(request_execution, &runtime, "service");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: "service-v2".to_string(),
                desired_aspect_fields: AspectFieldPatch::from_locator(
                    name_patch_target(),
                    AspectValue::String(InternedString::Raw("service".to_string())),
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

    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("decode output");
    assert_eq!(
        output.action,
        EntityReplacementReconciliationAction::ReplaceEntity
    );
}

mod unchanged_state;
