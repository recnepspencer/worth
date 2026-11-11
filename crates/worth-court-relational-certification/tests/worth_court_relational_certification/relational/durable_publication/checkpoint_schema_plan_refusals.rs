use super::branch_history_publications::create_with_fields;
use super::branch_history_publications::relation_with_fields;
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::config::CrossContextPolicy;
use worth_relational::facade::durability::RecoveryAuthorityContinuityMismatch;
use worth_relational::facade::durability::RecoveryFailureClass;
use worth_relational::facade::durability::RecoveryVerificationMode as RecoveryMode;
use worth_relational::facade::durability::RelationIntegrityContractFamily;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationKindRegistration;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_testkit_relational::aspect_storage::definition::aspect_key;
use worth_testkit_relational::aspect_storage::definition::field_key;
use worth_testkit_relational::aspect_storage::entity_field_aspect;

use worth_testkit_relational::aspect_storage::relation_field_aspect;

use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::create_entity_outcome;

#[test]
fn durability_contract_failure_aspect_plan_mismatch_is_explicit() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
            .build_registry(),
        )
        .build_durable(medium.attachment())
        .unwrap();
    create_entity_outcome(execution, &runtime, "main-a");
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();
    let expected_registry = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let mismatched_registry = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("display_name"), field_key("name")),
            super::merge_witness_schema::materialize(
                worth_testkit_relational::aspect_storage::definition::lifecycle_aspect(),
            ),
        ],
        relation_aspects: vec![
            relation_field_aspect(aspect_key("label"), field_key("label")),
            super::merge_witness_schema::materialize(
                worth_testkit_relational::aspect_storage::definition::lifecycle_aspect(),
            ),
            super::merge_witness_schema::materialize(
                worth_testkit_relational::aspect_storage::definition::relation_source_aspect(),
            ),
            super::merge_witness_schema::materialize(
                worth_testkit_relational::aspect_storage::definition::relation_target_aspect(),
            ),
        ],
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    let expected_revision = expected_registry
        .entity_aspect_declaration_trace(KindId(1))
        .unwrap()
        .plan_revision;
    let mismatched_revision = mismatched_registry
        .entity_aspect_declaration_trace(KindId(1))
        .unwrap()
        .plan_revision;
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .build()
        .expect("valid native runtime configuration");
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = super::checkpoint_recovery_observation::failure(error);

    assert_ne!(expected_revision, mismatched_revision);
    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::EntityAspectPlanRevision {
            kind_id: KindId(1),
            expected_revision: expected,
            found_revision: found,
            ..
        }) if expected == expected_revision.0 && found == mismatched_revision.0
    ));
}

#[test]
fn durability_contract_failure_relation_integrity_plan_mismatch_is_explicit() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let base_registry = RelationalSchemaRegistry::new()
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
                relation_integrity: worth_relational::facade::schema::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![worth_relational::facade::schema::CardinalityContractDeclaration {
                        contract_id: "source_max_one".into(),
                        source_max: Some(1),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: worth_relational::facade::schema::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            worth_relational::facade::schema::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(base_registry)
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration");
    let main = worth_relational::facade::history::BranchId("main".into());
    let source_commit =
        create_with_fields(execution, &runtime, "source", &main, Default::default());
    let target_commit =
        create_with_fields(execution, &runtime, "target", &main, Default::default());
    let source =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&source_commit)[0];
    let target =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&target_commit)[0];
    for commit in [&source_commit, &target_commit] {
        worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot(
            &runtime, commit,
        );
    }
    relation_with_fields(
        execution,
        &runtime,
        source,
        target,
        "guarded",
        Default::default(),
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .unwrap();

    let mismatched_registry = RelationalSchemaRegistry::new()
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
                relation_integrity: worth_relational::facade::schema::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![worth_relational::facade::schema::CardinalityContractDeclaration {
                        contract_id: "source_max_two".into(),
                        source_max: Some(2),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics: worth_relational::facade::schema::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            worth_relational::facade::schema::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(mismatched_registry)
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration");
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = super::checkpoint_recovery_observation::failure(error);

    assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch);
    assert!(matches!(
        error.authority_continuity_mismatch,
        Some(RecoveryAuthorityContinuityMismatch::RelationIntegrityPlanRevision {
            kind_id: KindId(2),
            contract_family: RelationIntegrityContractFamily::Cardinality,
            ref expected_contract_ids,
            ref found_contract_ids,
            ..
        }) if expected_contract_ids == &vec![worth_relational::facade::schema::ContractId::from("source_max_one")]
            && found_contract_ids == &vec![worth_relational::facade::schema::ContractId::from("source_max_two")]
    ));
}
