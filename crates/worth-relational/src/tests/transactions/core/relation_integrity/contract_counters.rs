use crate::tests::support::*;

#[test]
fn relation_integrity_commit_reports_contract_counters_on_success() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    vec![crate::schema::data::EndpointKindContractDeclaration {
                        contract_id: "no_self".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    vec![crate::schema::data::CardinalityContractDeclaration {
                        contract_id: "source_max_two".into(),
                        source_max: Some(2),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    vec![crate::schema::data::UniquenessContractDeclaration {
                        contract_id: "uniq".into(),
                        scope: crate::schema::data::UniquenessScope::DirectedSemanticEdge,
                    }],
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");

    let result = create_relation_outcome(execution, &runtime, source, target, "guarded");

    assert!(
        result
            .complexity_delta()
            .relation_integrity_contracts_evaluated
            >= 3
    );
    assert!(result.complexity_delta().relation_endpoint_kind_checks >= 1);
    assert!(result.complexity_delta().relation_cardinality_checks >= 1);
    assert!(result.complexity_delta().relation_uniqueness_checks >= 1);
}
