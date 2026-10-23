//! One structural-surface world per installed rule set, so a shared run can be
//! compared with the same rules installed alone.
use super::super::validation_engine_fixtures::*;
use crate::identity::data::EntityId;
use crate::performance::data::RuntimeComplexityCounters;
use crate::validation::engine::InvariantExecutionResult;

pub(super) struct StructuralWorld {
    runtime: RelationalRuntime,
    source: EntityId,
    target: EntityId,
    plan: MergedCommitPlan,
}

impl StructuralWorld {
    pub(super) fn install(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        rules: impl IntoIterator<Item = CustomInvariantRegistration>,
    ) -> Self {
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
        let runtime = rules
            .into_iter()
            .fold(
                RelationalRuntimeApi::builder().schema_registry(schema),
                |builder, rule| builder.custom_invariant(rule),
            )
            .build();
        let source = create_entity_of_kind(execution, &runtime, KindId(1), "structural-source");
        let target = create_entity_of_kind(execution, &runtime, KindId(1), "structural-target");
        let relation = |client_key: &str| {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: crate::facade::identity::KindId(2),
                client_key: ClientKey::raw(client_key),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }))
        };
        let plan = MergedCommitPlan {
            transaction_id: TransactionId(3),
            merged_intents: vec![
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(1),
                    client_key: ClientKey::raw("source"),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                })),
                relation("edge"),
                relation("edge-2"),
            ],
        };
        Self {
            runtime,
            source,
            target,
            plan,
        }
    }

    /// Run the planned candidate at the commit boundary from zeroed counters.
    pub(super) fn evaluate(&self) -> (InvariantExecutionResult, RuntimeComplexityCounters) {
        self.runtime.performance_access().reset_counters();
        let results = InvariantEngine::new(&self.runtime).execute_fixture(
            InvariantExecutionRequest::from_profile_with_contract(
                InvariantRequestProfile::CommitBoundary,
                &self.runtime,
                InvariantObservation::committed(self.runtime.storage_access().current_edition()),
                self.runtime.current_version_id(),
                Some(&self.plan),
                None,
            ),
        );
        (results, self.runtime.performance_access().counters())
    }

    pub(super) fn commit_edge(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) {
        create_relation_of_kind(
            execution,
            &self.runtime,
            KindId(2),
            self.source,
            self.target,
            "committed-edge",
        );
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
