pub(super) use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
pub(super) use crate::identity::data::KindId;
pub(super) use crate::identity::data::PartitionId;
pub(super) use crate::identity::data::RelationId;
pub(super) use crate::schema::data::EntityKindRegistration;
pub(super) use crate::schema::data::KindAspectContractDeclarations;
pub(super) use crate::schema::data::RelationKindRegistration;
pub(super) use crate::schema::data::RelationalSchemaRegistry;
pub(super) use crate::schema::data::SchemaId;
pub(super) use crate::schema::data::SchemaVersionId;
pub(super) use crate::schema::data::{
    AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
    ConnectivityMinimumEnforcement, DirectedTraversalKind, PartitionIsolationContractDeclaration,
    PartitionIsolationMode, RelationIntegrityDeclarations,
};
pub(super) use crate::symbols::data::ClientKey;
pub(super) use crate::transactions::data::intents::mutation_intent::CreateIntent;
pub(super) use crate::transactions::data::intents::mutation_intent::DeleteRelationIntent;
pub(super) use crate::transactions::data::intents::mutation_intent::MutationIntent;
pub(super) use crate::transactions::data::intents::mutation_intent::RelationMutationIntent;
pub(super) use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
pub(super) use crate::transactions::data::primitives::EntitySpec;
pub(super) use crate::transactions::data::primitives::RelationSpec;
pub(super) use crate::transactions::data::primitives::TransactionId;
pub(super) use crate::transactions::data::primitives::WorkerIntentBatch;
pub(super) use crate::validation::custom_rule::CustomInvariantExecutionContext;
pub(super) use crate::validation::custom_rule::CustomInvariantRegistration;
pub(super) use crate::validation::custom_rule::CustomInvariantRule;
pub(super) use crate::validation::custom_rule::CustomInvariantScopePlanner;
pub(super) use crate::validation::data::catalog::InvariantCatalog;
pub(super) use crate::validation::data::catalog::InvariantRegistration;
pub(super) use crate::validation::data::custom_rule::errors::CustomInvariantExecutionError;
pub(super) use crate::validation::data::custom_rule::errors::CustomInvariantPreparationError;
pub(super) use crate::validation::data::custom_rule::errors::CustomInvariantVerdict;
pub(super) use crate::validation::data::descriptor::CustomInvariantDescriptor;
pub(super) use crate::validation::data::descriptor::CustomInvariantOperationalMetadata;
pub(super) use crate::validation::data::execution::InvariantReportedRule;
pub(super) use crate::validation::data::groups::InvariantGroup;
pub(super) use crate::validation::data::groups::InvariantGroupSet;
pub(super) use crate::validation::data::results::InvariantViolationFields;
pub(super) use crate::validation::data::rule_id::CustomInvariantRuleId;
pub(super) use crate::validation::data::rule_id::CustomInvariantSemanticIdentity;
pub(super) use crate::validation::data::rule_id::CustomInvariantSemanticVersion;
pub(super) use crate::validation::data::rules::InvariantRule;
pub(super) use crate::validation::engine::{
    InvariantEngine, InvariantExecutionRequest, InvariantObservation, InvariantRequestProfile,
};

pub(super) fn partition_isolation_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
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
        .unwrap()
}

pub(super) fn cardinality_minimum_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
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
        .unwrap()
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
