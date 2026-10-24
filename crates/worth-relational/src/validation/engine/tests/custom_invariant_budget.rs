use super::custom_rules::*;
use super::validation_engine_fixtures::*;

#[test]
fn unrelated_custom_rule_returns_owner_proven_non_applicability() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap())
        .build();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(900),
        merged_intents: Vec::new(),
    };
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::NotApplicable
    );
    assert_eq!(
        runtime
            .performance_access()
            .counters()
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
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(CustomInvariantRegistration::new(TinyBudgetUnrelatedRule).unwrap())
        .build();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(901),
        merged_intents: ["first", "second"]
            .into_iter()
            .map(|key| {
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(key),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }))
            })
            .collect(),
    };
    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::NotApplicable
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
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(CustomInvariantRegistration::new(BoundedBudgetUnrelatedRule).unwrap())
        .build();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(902),
        merged_intents: (0..32)
            .map(|index| {
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(format!("entity-{index}")),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                }))
            })
            .collect(),
    };
    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::NotApplicable
    );
    assert_eq!(
        runtime
            .performance_access()
            .counters()
            .custom_invariant_execution_count,
        0
    );
}

#[test]
fn unrelated_bulk_relation_endpoint_scan_cannot_escape_work_budget() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "unrelated.entity".to_owned(),
            schema_id: SchemaId("unrelated".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(CustomInvariantRegistration::new(BoundedBudgetUnrelatedRule).unwrap())
        .build();
    let source = crate::tests::support::create_entity(execution, &runtime, "bulk-source");
    let target = crate::tests::support::create_entity(execution, &runtime, "bulk-target");
    let endpoints = (0..40)
        .map(|_| {
            (
                crate::transactions::data::EntityReference::Existing(source),
                crate::transactions::data::EntityReference::Existing(target),
            )
        })
        .collect::<Vec<_>>();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(903),
        merged_intents: vec![MutationIntent::Create(CreateIntent::BulkRelations(
            crate::transactions::data::BulkRelationCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_keys: (0..endpoints.len())
                    .map(|index| ClientKey::raw(format!("relation-{index}")))
                    .collect(),
                field_patches: vec![
                    crate::transactions::data::AspectFieldPatch::default();
                    endpoints.len()
                ],
                endpoints,
            },
        ))],
    };
    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
    assert_eq!(
        runtime
            .performance_access()
            .counters()
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
        let entity = crate::identity::data::EntityId::new(PartitionId::main(), 1, 1);
        while context
            .relations()
            .outgoing_relations_for_entity(entity)
            .is_ok()
        {}
        Ok(CustomInvariantVerdict::Violation(
            crate::validation::data::CustomInvariantRefusal::Empty,
        ))
    }
}

#[test]
fn rule_overrunning_its_installed_work_is_an_execution_error_not_a_violation() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(SwallowsWorkExhaustionRule).unwrap())
        .build();
    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::validation::data::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("an overrun rule must fail closed");
    };
    let crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
        identity,
        phase,
        failure,
        detail,
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
        &crate::validation::data::CustomInvariantFailurePhase::Execution
    );
    assert_eq!(
        failure,
        &crate::validation::data::ResultCustomInvariantFailureKind::ExecutionError
    );
    assert_eq!(
        detail,
        "custom invariant exceeded its installed work budget"
    );
    assert_eq!(results.summary().custom_failure_count(), 1);
}
