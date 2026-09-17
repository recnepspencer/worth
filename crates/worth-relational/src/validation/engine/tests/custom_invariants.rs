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
fn exhausted_applicability_scan_cannot_claim_non_applicability() {
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
                .into()
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
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn engine_executes_custom_invariant_packets() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap())
        .build();

    let results = InvariantEngine::new(&runtime).execute(
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
    match &results.results()[0].rule {
        InvariantReportedRule::Custom(identity) => {
            assert_eq!(identity.rule_id.as_str(), "test.custom.violation");
        }
        other => panic!("expected custom invariant result, got {other:?}"),
    }
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn engine_executes_graph_composition_custom_invariant_packets() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(GraphCompositionViolatesCustomRule).unwrap(),
        )
        .build();

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::GraphComposition,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].execution_point,
        crate::validation::data::InvariantExecutionPoint::GraphComposition
    );
    assert_eq!(
        results.metadata().max_cost(),
        crate::validation::data::InvariantCostClass::Touched
    );
    match &results.results()[0].rule {
        InvariantReportedRule::Custom(identity) => {
            assert_eq!(identity.rule_id.as_str(), "test.custom.graph-composition");
        }
        other => panic!("expected graph composition custom invariant result, got {other:?}"),
    }
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn engine_executes_custom_packets_against_real_structural_surfaces() {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "custom.entity".to_owned(),
            schema_id: SchemaId("custom".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|schema| {
            schema.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "custom.relation".to_owned(),
                schema_id: SchemaId("custom".to_owned()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("the structural fixture schema should register");
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .custom_invariant(CustomInvariantRegistration::new(StructuralSurfaceRule).unwrap())
        .build();
    let source = create_entity_of_kind(&runtime, KindId(1), "structural-source");
    let target = create_entity_of_kind(&runtime, KindId(1), "structural-target");
    runtime.performance_access().reset_counters();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(3),
        merged_intents: vec![
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: crate::facade::identity::KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("source"),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }))
            .into(),
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: crate::facade::identity::KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("edge"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }))
            .into(),
        ],
    };

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()).into(),
            runtime.current_version_id(),
            Some(&plan),
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 1);
    assert!(counters.custom_invariant_traversal_frontier_count >= 2);
}

#[test]
fn engine_captures_custom_prepare_panics_as_typed_failures() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(PanicDuringPrepareRule).unwrap())
        .build();
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute(
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
        panic!("expected captured prepare panic to produce a violation");
    };
    match &violation.fields {
        crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => {
            assert_eq!(
                identity.semantic_identity().rule_id.as_str(),
                "test.custom.panic-prepare"
            );
            assert_eq!(
                phase,
                &crate::validation::data::CustomInvariantFailurePhase::Preparation
            );
            assert_eq!(
                failure,
                &crate::validation::data::ResultCustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 0);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}

#[test]
fn engine_captures_custom_evaluate_panics_as_typed_failures() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(PanicDuringEvaluateRule).unwrap())
        .build();
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute(
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
        panic!("expected captured evaluate panic to produce a violation");
    };
    match &violation.fields {
        crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => {
            assert_eq!(
                identity.semantic_identity().rule_id.as_str(),
                "test.custom.panic-evaluate"
            );
            assert_eq!(
                phase,
                &crate::validation::data::CustomInvariantFailurePhase::Execution
            );
            assert_eq!(
                failure,
                &crate::validation::data::ResultCustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 1);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}
