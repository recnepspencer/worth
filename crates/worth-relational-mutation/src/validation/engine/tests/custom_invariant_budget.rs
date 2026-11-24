use super::custom_rules::*;
use super::validation_engine_fixtures::*;

#[test]
fn unrelated_custom_rule_returns_owner_proven_non_applicability() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(900),
        merged_intents: Vec::new(),
    };

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::NotApplicable
    );
    assert_eq!(
        instrumentation
            .complexity_snapshot()
            .custom_invariant_execution_count,
        0
    );
}

#[test]
fn complete_kind_footprint_proves_non_applicability_before_rule_budget() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "unrelated.entity".to_owned(),
            schema_id: SchemaId("unrelated".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = schema;
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(TinyBudgetUnrelatedRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(901),
        merged_intents: ["first", "second"]
            .into_iter()
            .map(|key| {
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::contracts::symbols::data::client_key::ClientKey::raw(key),
                    fields:
                        crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
                }))
            })
            .collect(),
    };
    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::NotApplicable
    );
}

#[test]
fn unrelated_candidate_records_do_not_consume_restricted_scope_budget() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "unrelated.entity".to_owned(),
            schema_id: SchemaId("unrelated".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = schema;
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(BoundedBudgetUnrelatedRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(902),
        merged_intents: (0..32)
            .map(|index| {
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::contracts::symbols::data::client_key::ClientKey::raw(format!("entity-{index}")),
                    fields:
                        crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
                }))
            })
            .collect(),
    };
    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::NotApplicable
    );
    assert_eq!(
        instrumentation
            .complexity_snapshot()
            .custom_invariant_execution_count,
        0
    );
}

#[test]
fn unrelated_bulk_relation_endpoint_scan_cannot_escape_work_budget() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "unrelated.entity".to_owned(),
            schema_id: SchemaId("unrelated".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let partition_id = PartitionId::main();
    let (partition, entities, _) =
        worth_relational_storage::internal_port::storage::overlay::small_relation_partition::relation_partition_for_test(
            partition_id,
            2,
            &[],
        );
    let partitions = std::collections::BTreeMap::from([(partition_id, partition)]);
    let source = entities[0];
    let target = entities[1];
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = schema;
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(1);
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(BoundedBudgetUnrelatedRule).unwrap()],
    );
    let endpoints = (0..40)
        .map(|_| {
            (
                crate::contracts::transactions::data::primitives::EntityReference::Existing(source),
                crate::contracts::transactions::data::primitives::EntityReference::Existing(target),
            )
        })
        .collect::<Vec<_>>();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(903),
        merged_intents: vec![MutationIntent::Create(CreateIntent::BulkRelations(
            crate::contracts::transactions::data::intents::mutation_intent::BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: (0..endpoints.len())
                    .map(|index| ClientKey::raw(format!("relation-{index}")))
                    .collect(),
                field_patches: vec![
                    crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default();
                    endpoints.len()
                ],
                endpoints,
            },
        ))],
    };
    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert!(matches!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::Violation(_)
    ));
    assert_eq!(
        instrumentation
            .complexity_snapshot()
            .custom_invariant_execution_count,
        0
    );
}

/// Reads adjacency until its installed allowance refuses, then swallows the
/// refusal and reports a domain violation anyway.
struct SwallowsWorkExhaustionRule;

impl CustomInvariantRule for SwallowsWorkExhaustionRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = AlwaysViolatesCustomRule.descriptor();
        descriptor.identity.rule_id = CustomInvariantRuleId::new("test.custom.swallows-work");
        descriptor.operational.maximum_work_units = Some(std::num::NonZeroU64::new(4).unwrap());
        descriptor
    }

    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let entity =
            crate::contracts::identity::data::record_ids::EntityId::new(PartitionId::main(), 1, 1);
        while context
            .relations()
            .outgoing_relations_for_entity(entity)
            .is_ok()
        {}
        Ok(CustomInvariantVerdict::Violation(
            crate::contracts::validation::data::custom_rule::refusal::CustomInvariantRefusal::Empty,
        ))
    }
}

#[test]
fn rule_overrunning_its_installed_work_is_an_execution_error_not_a_violation() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(SwallowsWorkExhaustionRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::contracts::validation::data::execution::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("an overrun rule must fail closed");
    };
    let crate::contracts::validation::data::results::InvariantViolationFields::CustomInvariantFailure {
        identity,
        phase,
        failure,
        detail,
        ..
    } = &violation.fields
    else {
        panic!("the swallowed refusal must not reach the caller: {violation:?}");
    };
    assert_eq!(
        identity.semantic_identity().rule_id.as_str(),
        "test.custom.swallows-work"
    );
    assert_eq!(
        phase,
        &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase::Execution
    );
    assert_eq!(
        failure,
        &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::ExecutionError
    );
    assert_eq!(
        detail,
        "custom invariant exceeded its installed work budget"
    );
    assert_eq!(results.summary().custom_failure_count(), 1);
}
