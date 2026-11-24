//! One structural-surface world per installed rule set, so a shared run can be
//! compared with the same rules installed alone.
use super::super::validation_engine_fixtures::*;
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::VersionId;
use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::runtime::state::subsystems::{
    services::instrumentation::RuntimeInstrumentation,
    storage::{PartitionEditionCopyLane, StorageSubsystem},
};
use crate::storage::overlay::small_relation_partition::relation_partition_for_test;
use crate::validation::engine::test_view::invariant_view_with_custom_rules;
use crate::validation::engine::InvariantExecutionResult;
use std::cell::Cell;

pub(super) struct StructuralWorld {
    storage: StorageSubsystem,
    config: RelationalRuntimeConfig,
    rules: Vec<CustomInvariantRegistration>,
    version: Cell<VersionId>,
    plan: MergedCommitPlan,
}

impl StructuralWorld {
    pub(super) fn install(rules: impl IntoIterator<Item = CustomInvariantRegistration>) -> Self {
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
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = schema;
        let rules = rules.into_iter().collect();
        let partition_id = PartitionId::main();
        let (partition, entities, _) = relation_partition_for_test(partition_id, 2, &[]);
        let [source, target] = entities[..] else {
            unreachable!("the lower owner admitted two entity slots");
        };
        let storage = StorageSubsystem::default();
        storage.install_owned(
            &Default::default(),
            PartitionEditionCopyLane::Ordinary,
            [(partition_id, partition)],
        );
        let relation = |client_key: &str| {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: crate::identity::data::KindId(2),
                client_key: ClientKey::raw(client_key),
                source: crate::transactions::data::primitives::EntityReference::Existing(source),
                target: crate::transactions::data::primitives::EntityReference::Existing(target),
                fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
            }))
        };
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(3),
            merged_intents: vec![
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: crate::identity::data::KindId(1),
                    client_key: ClientKey::raw("source"),
                    fields:
                        crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
                })),
                relation("edge"),
                relation("edge-2"),
            ],
        };
        Self {
            storage,
            config,
            rules,
            version: Cell::new(VersionId(2)),
            plan,
        }
    }

    /// Run the planned candidate at the commit boundary from zeroed counters.
    pub(super) fn evaluate(&self) -> (InvariantExecutionResult, RuntimeComplexityCounters) {
        let instrumentation = RuntimeInstrumentation::default();
        let version = self.version.get();
        let view = invariant_view_with_custom_rules(
            &instrumentation,
            self.config.clone(),
            version,
            self.rules.clone(),
        );
        let results = InvariantEngine::from_view(&view).execute_fixture(
            InvariantExecutionRequest::from_profile_with_contract(
                InvariantRequestProfile::CommitBoundary,
                &view,
                InvariantObservation::committed(self.storage.acquire()),
                version,
                Some(&self.plan),
                None,
            ),
        );
        (results, instrumentation.complexity_snapshot())
    }

    pub(super) fn install_committed_edge(&self) {
        let partition_id = PartitionId::main();
        let (partition, _, _) = relation_partition_for_test(partition_id, 2, &[(0, 1, KindId(2))]);
        self.storage.install_owned(
            &Default::default(),
            PartitionEditionCopyLane::Ordinary,
            [(partition_id, partition)],
        );
        self.version.set(VersionId(3));
    }
}

/// Every candidate-input lookup is answered by exactly one physical
/// materialization or one reuse hit. Adjacency relation ids measure a gather's
/// size, not a lookup, so they are not part of the sum.
pub(super) fn candidate_lookups(counters: &RuntimeComplexityCounters) -> usize {
    [
        counters.custom_invariant_candidate_entity_reads,
        counters.custom_invariant_candidate_relation_reads,
        counters.custom_invariant_candidate_entity_aspect_reads,
        counters.custom_invariant_candidate_relation_aspect_reads,
        counters.custom_invariant_candidate_adjacency_gathers,
        counters.custom_invariant_candidate_adjacency_count_reads,
        counters.custom_invariant_candidate_touched_entity_gathers,
        counters.custom_invariant_candidate_touched_relation_gathers,
        counters.custom_invariant_candidate_touched_partition_gathers,
        counters.custom_invariant_candidate_touched_entity_slot_gathers,
        counters.custom_invariant_candidate_touched_relation_slot_gathers,
        counters.custom_invariant_candidate_reuse_hits,
    ]
    .into_iter()
    .sum()
}
