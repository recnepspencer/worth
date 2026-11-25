pub(super) use crate::contracts::config::data::policies::CascadeDeletePolicy;
pub(super) use crate::contracts::config::data::policies::CrossContextPolicy;
pub use crate::contracts::identity::data::primitives::KindId;
pub use crate::contracts::identity::data::primitives::PartitionId;
#[cfg(test)]
pub(super) use crate::contracts::identity::data::record_ids::RelationId;
pub(super) use crate::contracts::schema::data::aspect_contracts::kind_declarations::KindAspectContractDeclarations;
pub(super) use crate::contracts::schema::data::relation_integrity::declarations::RelationIntegrityDeclarations;
pub(super) use crate::contracts::schema::data::structural_invariants::AcyclicityContractDeclaration;
pub(super) use crate::contracts::schema::data::structural_invariants::AllowedCycleClass;
pub(super) use crate::contracts::schema::data::structural_invariants::ConnectivityMinimumContractDeclaration;
pub(super) use crate::contracts::schema::data::structural_invariants::ConnectivityMinimumEnforcement;
pub(super) use crate::contracts::schema::data::structural_invariants::DirectedTraversalKind;
pub(super) use crate::contracts::schema::data::structural_invariants::PartitionIsolationContractDeclaration;
pub(super) use crate::contracts::schema::data::structural_invariants::PartitionIsolationMode;
pub(super) use crate::contracts::schema::data::EntityKindRegistration;
pub(super) use crate::contracts::schema::data::RelationKindRegistration;
pub(super) use crate::contracts::schema::data::RelationalSchemaRegistry;
pub(super) use crate::contracts::schema::data::SchemaId;
pub(super) use crate::contracts::schema::data::SchemaVersionId;
pub use crate::contracts::symbols::data::client_key::ClientKey;
pub use crate::contracts::transactions::data::intents::mutation_intent::CreateIntent;
#[cfg(test)]
pub(super) use crate::contracts::transactions::data::intents::mutation_intent::DeleteRelationIntent;
pub use crate::contracts::transactions::data::intents::mutation_intent::MutationIntent;
#[cfg(test)]
pub(super) use crate::contracts::transactions::data::intents::mutation_intent::RelationMutationIntent;
#[cfg(test)]
pub(super) use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
pub use crate::contracts::transactions::data::primitives::EntitySpec;
pub(super) use crate::contracts::transactions::data::primitives::RelationSpec;
#[cfg(test)]
pub(super) use crate::contracts::transactions::data::primitives::TransactionId;
pub use crate::contracts::transactions::data::primitives::WorkerIntentBatch;
pub use crate::contracts::validation::data::catalog::InvariantCatalog;
pub use crate::contracts::validation::data::catalog::InvariantRegistration;
pub(super) use crate::contracts::validation::data::custom_rule::errors::CustomInvariantExecutionError;
pub(super) use crate::contracts::validation::data::custom_rule::errors::CustomInvariantPreparationError;
pub(super) use crate::contracts::validation::data::custom_rule::errors::CustomInvariantVerdict;
pub(super) use crate::contracts::validation::data::descriptor::CustomInvariantDescriptor;
pub(super) use crate::contracts::validation::data::descriptor::CustomInvariantOperationalMetadata;
pub use crate::contracts::validation::data::execution::InvariantReportedRule;
pub(super) use crate::contracts::validation::data::groups::InvariantGroup;
pub(super) use crate::contracts::validation::data::groups::InvariantGroupSet;
pub use crate::contracts::validation::data::results::InvariantViolationFields;
pub(super) use crate::contracts::validation::data::rule_id::CustomInvariantRuleId;
pub(super) use crate::contracts::validation::data::rule_id::CustomInvariantSemanticIdentity;
pub(super) use crate::contracts::validation::data::rule_id::CustomInvariantSemanticVersion;
pub use crate::contracts::validation::data::rules::InvariantRule;
pub(super) use crate::validation::custom_rule::CustomInvariantExecutionContext;
pub use crate::validation::custom_rule::CustomInvariantRegistration;
pub(super) use crate::validation::custom_rule::CustomInvariantRule;
pub(super) use crate::validation::custom_rule::CustomInvariantScopePlanner;
#[cfg(test)]
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
                    vec![crate::contracts::schema::data::relation_integrity::declarations::EndpointKindContractDeclaration {
                        contract_id: "node_domains".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    ["min_one", "min_one_again"]
                        .into_iter()
                        .map(|contract_id| crate::contracts::schema::data::relation_integrity::declarations::CardinalityContractDeclaration {
                            contract_id: contract_id.into(),
                            source_max: None,
                            source_min: Some(1),
                            target_max: None,
                            target_min: None,
                            pair_max: None,
                            pair_min: None,
                            pair_min_semantics: crate::contracts::schema::data::relation_integrity::declarations::PairMinimumSemantics::ObservedDirectedPairs,
                            minimum_enforcement:
                                crate::contracts::schema::data::relation_integrity::declarations::MinimumCardinalityEnforcement::CertificationBoundary,
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

pub fn acyclicity_and_connectivity_registry() -> RelationalSchemaRegistry {
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
