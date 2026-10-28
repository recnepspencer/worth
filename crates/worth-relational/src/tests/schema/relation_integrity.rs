use super::super::support::*;

fn relation_integrity_registry(
    relation_integrity: crate::schema::data::RelationIntegrityDeclarations,
) -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
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
                relation_integrity,
            })
        })
        .unwrap()
}

#[test]
fn relation_integrity_declaration_lowering_is_stable() {
    let registry =
        relation_integrity_registry(crate::schema::data::RelationIntegrityDeclarations::new(
            vec![crate::schema::data::EndpointKindContractDeclaration {
                contract_id: "endpoint".into(),
                allowed_source_kinds: vec![KindId(1)],
                allowed_target_kinds: vec![KindId(1)],
                self_edges_allowed: false,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
            }],
            vec![crate::schema::data::CardinalityContractDeclaration {
                contract_id: "source_cardinality".into(),
                source_max: Some(1),
                source_min: None,
                target_max: None,
                target_min: None,
                pair_max: None,
                pair_min: None,
                pair_min_semantics:
                    crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement:
                    crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
            }],
            vec![crate::schema::data::UniquenessContractDeclaration {
                contract_id: "uniq".into(),
                scope: crate::schema::data::UniquenessScope::DirectedSemanticEdge,
            }],
            vec![crate::schema::data::SymmetryContractDeclaration {
                contract_id: "symmetry".into(),
                mode: crate::schema::data::SymmetryMode::InverseProhibited,
            }],
            vec![crate::schema::data::EndpointDeletionIntegrityDeclaration {
            contract_id: "delete_guard".into(),
            mode: crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
        }],
        ));

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let plan = runtime.relation_integrity_plan(KindId(2)).unwrap();

    assert_eq!(plan.kind_id, KindId(2));
    assert_eq!(plan.contract_count(), 5);
    assert_ne!(plan.plan_revision.0, 0);
    assert_eq!(plan.endpoint_kind_contracts[0].contract_id, "endpoint");
    assert_eq!(plan.cardinality_maximum_contracts[0].source_max, Some(1));
    assert!(plan.cardinality_minimum_contracts.is_empty());
}

#[test]
fn relation_integrity_minimum_cardinality_lowering_is_publication_explicit() {
    let registry =
        relation_integrity_registry(crate::schema::data::RelationIntegrityDeclarations::new(
            vec![crate::schema::data::EndpointKindContractDeclaration {
                contract_id: "endpoint".into(),
                allowed_source_kinds: vec![KindId(1)],
                allowed_target_kinds: vec![KindId(1)],
                self_edges_allowed: false,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
            }],
            vec![crate::schema::data::CardinalityContractDeclaration {
                contract_id: "source_minimum".into(),
                source_max: None,
                source_min: Some(1),
                target_max: None,
                target_min: None,
                pair_max: None,
                pair_min: None,
                pair_min_semantics:
                    crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement:
                    crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
            }],
            vec![],
            vec![],
            vec![],
        ));

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let plan = runtime.relation_integrity_plan(KindId(2)).unwrap();
    let minimum = &plan.cardinality_minimum_contracts[0];

    assert_eq!(plan.cardinality_maximum_contracts.len(), 0);
    assert_eq!(minimum.contract_id, "source_minimum");
    assert_eq!(minimum.source_min, Some(1));
    assert_eq!(
        minimum.minimum_enforcement,
        crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary
    );
    assert_eq!(
        minimum.pair_min_semantics,
        crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs
    );
    assert_eq!(minimum.candidate_source_kinds, vec![KindId(1)]);
    assert_eq!(minimum.candidate_target_kinds, vec![KindId(1)]);
}

#[test]
fn duplicate_relation_contract_ids_are_rejected() {
    let error = RelationalSchemaRegistry::new()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "test.relation".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                vec![crate::schema::data::EndpointKindContractDeclaration {
                    contract_id: "dup".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(1)],
                    self_edges_allowed: true,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                }],
                vec![crate::schema::data::CardinalityContractDeclaration {
                    contract_id: "dup".into(),
                    source_max: Some(1),
                    source_min: None,
                    target_max: None,
                    target_min: None,
                    pair_max: None,
                    pair_min: None,
                    pair_min_semantics:
                        crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                    minimum_enforcement:
                        crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                }],
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        })
        .unwrap_err();

    assert!(matches!(
        error.class,
        crate::schema::data::SchemaRegistryErrorClass::DuplicateRelationContractId {
            kind_id: KindId(2),
            ..
        }
    ));
}

#[test]
fn schema_authority_recomputes_mutated_integrity_and_binds_map_key() {
    let registry =
        relation_integrity_registry(crate::schema::data::RelationIntegrityDeclarations::new(
            vec![],
            vec![crate::schema::data::CardinalityContractDeclaration {
                contract_id: "cardinality".into(),
                source_max: Some(1),
                source_min: None,
                target_max: None,
                target_min: None,
                pair_max: None,
                pair_min: None,
                pair_min_semantics:
                    crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement:
                    crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
            }],
            vec![],
            vec![],
            vec![],
        ));
    let baseline_digest = registry.authority_digest_bytes();
    let mut mutated = registry.clone();
    let registration = mutated
        .relation_kinds
        .get_mut(&KindId(2))
        .expect("relation kind is installed under its map key");
    registration.kind_id = KindId(999);
    registration.relation_integrity.cardinality_contracts[0].source_max = Some(2);

    assert_ne!(baseline_digest, mutated.authority_digest_bytes());
    assert_eq!(
        mutated.authority_snapshot().relation_kinds[0].kind_id,
        KindId(2),
        "the effective map key, not a mutable embedded label, is schema authority"
    );
    assert_ne!(
        registry.authority_snapshot().relation_kinds[0].relation_integrity_plan_revision,
        mutated.authority_snapshot().relation_kinds[0].relation_integrity_plan_revision,
        "mutated relation semantics cannot retain the stale plan revision"
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(mutated.clone())
        .build()
        .expect("valid native runtime configuration");
    assert_eq!(
        runtime
            .relation_integrity_plan(KindId(2))
            .expect("relation plan is lowered")
            .plan_revision,
        mutated.authority_snapshot().relation_kinds[0].relation_integrity_plan_revision,
        "lowered relation plans and schema roots share the effective revision"
    );
}

#[test]
fn relation_integrity_plan_revision_distinguishes_absent_and_max_u64_bounds() {
    let contract = |source_max| crate::schema::data::CardinalityContractDeclaration {
        contract_id: "cardinality".into(),
        source_max,
        source_min: None,
        target_max: None,
        target_min: None,
        pair_max: None,
        pair_min: None,
        pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
        minimum_enforcement:
            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
    };

    let absent = crate::schema::data::RelationIntegrityDeclarations::new(
        vec![],
        vec![contract(None)],
        vec![],
        vec![],
        vec![],
    );
    let explicit_max = crate::schema::data::RelationIntegrityDeclarations::new(
        vec![],
        vec![contract(Some(u64::MAX))],
        vec![],
        vec![],
        vec![],
    );

    assert_ne!(absent.plan_revision, explicit_max.plan_revision);
}
