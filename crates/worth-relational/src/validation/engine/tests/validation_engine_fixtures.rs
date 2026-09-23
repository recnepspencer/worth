pub(super) use crate::config::data::RelationIntegrityScopeBudget;
pub(super) use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
pub(super) use crate::facade::identity::{PartitionId, RelationId};
pub(super) use crate::facade::runtime::{InvariantCatalog, InvariantRegistration, InvariantRule};
pub(super) use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
pub(super) use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
pub(super) use crate::facade::transactions::{
    CreateIntent, DeleteRelationIntent, MergedCommitPlan, MutationIntent, RelationMutationIntent,
    TransactionId,
};
pub(super) use crate::identity::data::KindId;
pub(super) use crate::schema::data::{
    AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
    ConnectivityMinimumEnforcement, DirectedTraversalKind, PartitionIsolationContractDeclaration,
    PartitionIsolationMode, RelationIntegrityDeclarations,
};
pub(super) use crate::symbols::data::ClientKey;
pub(super) use crate::tests::support::{
    aspect_field_patch_from_values, aspect_key, entity_field_aspect, entity_summary_struct_aspect,
    field_key, string_aspect_value, AspectSchemaFixture,
};
pub(super) use crate::transactions::data::{EntitySpec, RelationSpec, WorkerIntentBatch};
pub(super) use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantGroup, InvariantGroupSet, InvariantReportedRule,
    InvariantViolationFields,
};
pub(super) use crate::validation::engine::{
    InvariantEngine, InvariantExecutionRequest, InvariantObservation, InvariantRequestProfile,
};

pub(super) fn runtime_with_invariants(invariant_catalog: InvariantCatalog) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .invariant_catalog(invariant_catalog)
        .build()
}

pub(super) fn runtime_with_partition_isolation() -> RelationalRuntime {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "geom.vertex".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "geom.edge".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default()
                    .with_partition_isolation_contracts(vec![
                        PartitionIsolationContractDeclaration {
                            contract_id: "same_partition".into(),
                            isolation_mode: PartitionIsolationMode::SamePartitionEndpoints,
                        },
                    ]),
            })
        })
        .unwrap();

    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

pub(super) fn runtime_with_cardinality_minimum() -> RelationalRuntime {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "geom.node".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "geom.edge".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![crate::schema::data::EndpointKindContractDeclaration {
                        contract_id: "node_domains".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    ["min_one", "min_one_again"]
                        .into_iter()
                        .map(|contract_id| crate::schema::data::CardinalityContractDeclaration {
                            contract_id: contract_id.into(),
                            source_max: None,
                            source_min: Some(1),
                            target_max: None,
                            target_min: None,
                            pair_max: None,
                            pair_min: None,
                            pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                            minimum_enforcement:
                                crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                        })
                        .collect(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();

    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

pub(super) fn acyclicity_and_connectivity_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "geom.node".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_entity_kind(EntityKindRegistration {
                kind_id: KindId(3),
                kind_name: "geom.anchor".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "geom.constraint".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default()
                    .with_acyclicity_contracts(vec![AcyclicityContractDeclaration {
                        contract_id: "no_cycles".into(),
                        traversal_direction: DirectedTraversalKind::SourceToTarget,
                        allowed_cycle_class: AllowedCycleClass::NoCycles,
                    }])
                    .with_connectivity_minimum_contracts(vec![
                        ConnectivityMinimumContractDeclaration {
                            contract_id: "reachable_anchor".into(),
                            source_kind_ids: vec![KindId(1)],
                            target_kind_ids: vec![KindId(3)],
                            minimum_reachable_targets: 1,
                            enforcement_boundary:
                                ConnectivityMinimumEnforcement::SnapshotPublication,
                        },
                    ]),
            })
        })
        .unwrap()
}

pub(super) fn runtime_with_acyclicity_and_connectivity() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .build()
}

pub(super) fn runtime_with_acyclicity_and_connectivity_budget(
    relation_integrity_scope_budget: RelationIntegrityScopeBudget,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .relation_integrity_scope_budget(relation_integrity_scope_budget)
        .build()
}

pub(super) fn create_entity_of_kind(
    runtime: &RelationalRuntime,
    kind_id: KindId,
    client_key: &str,
) -> crate::identity::data::EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("entity-{client_key}")).push(
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw(client_key),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }))
            .into(),
        ),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(runtime).expect("entity creation must succeed");
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity_id) => Some(*entity_id),
            crate::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("created entity id")
}

pub(super) fn create_relation_of_kind(
    runtime: &RelationalRuntime,
    kind_id: KindId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    client_key: &str,
) -> RelationId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("relation-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: ClientKey::raw(client_key),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(runtime).expect("relation creation must succeed");
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Relation(relation_id) => Some(*relation_id),
            crate::facade::transactions::RecordRef::Entity(_) => None,
        })
        .expect("created relation id")
}

pub(super) fn runtime_with_summary_title_uniqueness() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                entity_aspects: vec![
                    entity_field_aspect(aspect_key("name"), field_key("name")),
                    entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
                ],
                ..AspectSchemaFixture::default()
            }
            .build_registry(),
        )
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
                InvariantRule::unique_entity_aspect_field(
                    aspect_key("summary"),
                    field_key("title"),
                ),
            )],
            ..InvariantCatalog::default()
        })
        .build()
}

pub(super) fn runtime_with_summary_title_commit_boundary_uniqueness() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                entity_aspects: vec![
                    entity_field_aspect(aspect_key("name"), field_key("name")),
                    entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
                ],
                ..AspectSchemaFixture::default()
            }
            .build_registry(),
        )
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::unique_entity_aspect_field(
                    aspect_key("summary"),
                    field_key("title"),
                ),
            )],
            ..InvariantCatalog::default()
        })
        .build()
}

pub(super) fn commit_entity_with_summary(
    runtime: &RelationalRuntime,
    client_key: &str,
    title: &str,
    status: &str,
) -> Result<
    crate::facade::transactions::CommitResult,
    crate::transactions::data::TransactionCommitError,
> {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(WorkerIntentBatch::new(format!("entity-{client_key}")).push(
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(client_key),
            fields: aspect_field_patch_from_values([
                (
                    aspect_key("name"),
                    field_key("name"),
                    string_aspect_value(client_key),
                ),
                (
                    aspect_key("summary"),
                    field_key("title"),
                    string_aspect_value(title),
                ),
                (
                    aspect_key("summary"),
                    field_key("status"),
                    string_aspect_value(status),
                ),
            ]),
        })),
    ))
    .expect("test staging stays within configured resource budgets");
    txn.commit(runtime)
}
