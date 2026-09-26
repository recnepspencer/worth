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

    let results = InvariantEngine::new(&runtime).execute(
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
    let results = InvariantEngine::new(&runtime).execute(
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
    let results = InvariantEngine::new(&runtime).execute(
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
    let source = crate::tests::support::create_entity(&runtime, "bulk-source");
    let target = crate::tests::support::create_entity(&runtime, "bulk-target");
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
    let results = InvariantEngine::new(&runtime).execute(
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
