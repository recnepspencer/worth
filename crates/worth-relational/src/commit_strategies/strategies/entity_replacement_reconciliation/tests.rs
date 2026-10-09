use super::super::{
    EntityReplacementReconciliationAction, EntityReplacementReconciliationInput,
    EntityReplacementReconciliationOutput, EntityReplacementReconciliationStrategy,
};
use crate::commit_strategies::data::{
    StrategyCallerProvenance, StrategyExecutorFailureClass, StrategyRequestOrigin,
};
use crate::runtime::builder::RelationalRuntimeBuilder;
use crate::tests::support::{
    create_entity, entity_field_aspect, entity_u64_field_aspect, lifecycle_aspect,
    AspectSchemaFixture,
};
use crate::transactions::data::AspectFieldPatch;
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
    LocatorAuthority,
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

fn name_patch_target() -> AspectFieldLocator {
    crate::transactions::data::planned_single_field_locator(
        crate::tests::support::aspect_key("name"),
        crate::tests::support::field_key("name"),
    )
}

#[test]
fn entity_replacement_reconciliation_strategy_replaces_with_normalized_client_key_and_preserved_fields(
) {
    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::commit_strategies::data::CommitStrategyId(711),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            crate::commit_strategies::data::CommitStrategyRegistration::new(descriptor.clone())
                .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = create_entity(&runtime, "before");
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");
    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("output decode");
    let replacement = match &execution.mutation_program().worker_batches()[0].intents[0] {
        crate::transactions::data::MutationIntent::Entity(
            crate::transactions::data::EntityMutationIntent::Replace(intent),
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
        crate::symbols::data::ClientKey::raw("service-replacement")
    );
    assert_eq!(replacement_replicas, Some(&AspectValue::UInt64(3)));
}

#[test]
fn entity_replacement_reconciliation_strategy_replacement_declaration_applies_to_authority() {
    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::commit_strategies::data::CommitStrategyId(713),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            crate::commit_strategies::data::CommitStrategyRegistration::new(descriptor.clone())
                .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = create_entity(&runtime, "before");
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
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
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("replacement patch commit");
    let replacement_id = crate::tests::support::changed_entities(&commit)
        .into_iter()
        .last()
        .expect("replacement entity id");
    let replacement_snapshot = runtime.visibility_authority().snapshot();
    let replacement_read = runtime
        .visibility_reads()
        .read_snapshot(&replacement_snapshot)
        .expect("replacement snapshot should read");
    let replacement_record = replacement_read
        .get_entity(replacement_id)
        .expect("replacement record should be visible");

    let expected_replicas_key =
        crate::storage::data::authoritative_aspect_value_field_comparison_key(
            &AspectValue::UInt64(7),
        );
    let replicas_locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("replicas").expect("valid replicas aspect"),
        CanonicalFieldPath::single(FieldKey::new("replicas").expect("valid replicas field")),
    );
    assert_eq!(
        crate::visibility::materialization::read_records::entity_query_locus_comparison_key(
            replacement_record,
            &replicas_locator
        ),
        Some(expected_replicas_key)
    );
}

#[test]
fn entity_replacement_reconciliation_strategy_rejects_undeclared_fields() {
    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::commit_strategies::data::CommitStrategyId(712),
    );
    let registry = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(registry)
        .commit_strategy(
            crate::commit_strategies::data::CommitStrategyRegistration::new(descriptor.clone())
                .expect("strategy registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = create_entity(&runtime, "before");
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
    let snapshot = runtime.visibility_authority().snapshot();
    let error = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect_err("undeclared replacement fields should fail");

    match error {
        crate::commit_strategies::StrategyExecutionError::ExecutorFailed { failure, .. } => {
            assert_eq!(failure.class, StrategyExecutorFailureClass::DomainRejection);
            assert!(failure
                .detail
                .contains("not a lowered foundational scalar entity aspect"));
        }
        other => panic!("expected executor failure, got {other:?}"),
    }
}

#[test]
fn entity_replacement_reconciliation_strategy_replaces_when_only_client_key_changes() {
    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(77),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            EntityReplacementReconciliationStrategy::registration(descriptor.id())
                .expect("registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = create_entity(&runtime, "service");
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");

    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("decode output");
    assert_eq!(
        output.action,
        EntityReplacementReconciliationAction::ReplaceEntity
    );
}

#[test]
fn entity_replacement_reconciliation_strategy_noops_when_authoritative_fields_match() {
    let descriptor = EntityReplacementReconciliationStrategy::descriptor(
        crate::facade::commit_strategies::CommitStrategyId(78),
    );
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_registry())
        .commit_strategy(
            EntityReplacementReconciliationStrategy::registration(descriptor.id())
                .expect("registration"),
        )
        .commit_strategy_executor(
            EntityReplacementReconciliationStrategy::execution_registration(&descriptor),
        )
        .build();
    let entity = create_entity(&runtime, "service");
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &EntityReplacementReconciliationInput {
                entity_id: entity,
                replacement_client_key: String::new(),
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
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot)
        .expect("strategy execution");

    let output =
        EntityReplacementReconciliationOutput::decode(execution.output().canonical_bytes())
            .expect("decode output");
    assert_eq!(
        output.action,
        EntityReplacementReconciliationAction::NoChange
    );
    assert_eq!(execution.mutation_program().total_intent_count(), 0);
}
