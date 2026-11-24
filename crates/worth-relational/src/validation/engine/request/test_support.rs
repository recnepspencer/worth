use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy, RelationalRuntimeConfig};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{
    EndpointKindContractDeclaration, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use crate::storage::overlay::{small_relation_partition::relation_partition, PartitionState};
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::validation::data::contracts::InvariantPlanContract;
use crate::validation::engine::{
    test_view::invariant_view, InvariantExecutionRequest, InvariantObservation,
    InvariantPreparationControl, InvariantRequestProfile, InvariantRuntimeView,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;

fn schema() -> RelationalSchemaRegistry {
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
                kind_name: "test.edge.a".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind2".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(3),
                kind_name: "test.edge.b".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind3".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap()
}

pub(crate) struct RequestGraph {
    partitions: BTreeMap<PartitionId, PartitionState>,
    pub(crate) entities: Vec<EntityId>,
    pub(super) relations: Vec<RelationId>,
}

impl RequestGraph {
    pub(crate) fn new(entity_count: usize, edges: &[(usize, usize, KindId)]) -> Self {
        let id = PartitionId::main();
        let (partition, entities, relations) = relation_partition(id, entity_count, edges);
        Self {
            partitions: BTreeMap::from([(id, partition)]),
            entities,
            relations,
        }
    }

    pub(super) fn suspend(&mut self, relation: RelationId, endpoints: (EntityId, EntityId)) {
        let partition = self.partitions.get_mut(&relation.partition_id).unwrap();
        partition
            .relation_arena
            .suspend_materialization(relation.slot_index())
            .unwrap();
        partition
            .adjacency
            .get_mut(endpoints.0.slot_index())
            .unwrap()
            .remove(KindId(2), &relation);
        partition
            .reverse_adjacency
            .get_mut(endpoints.1.slot_index())
            .unwrap()
            .remove(KindId(2), &relation);
    }

    pub(crate) fn view(instrumentation: &RuntimeInstrumentation) -> InvariantRuntimeView<'_> {
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = schema();
        invariant_view(instrumentation, config, VersionId(1))
    }

    pub(crate) fn request<'state>(
        &'state self,
        view: &InvariantRuntimeView<'_>,
        plan: Option<&'state MergedCommitPlan>,
    ) -> InvariantExecutionRequest<'state> {
        let host = crate::execution::test_policy::serial_request();
        InvariantExecutionRequest::from_profile_with_contract_at_current_version(
            &ExecutionRequest::serial(&host),
            InvariantRequestProfile::CommitBoundary,
            view,
            InvariantObservation::committed_branch(&self.partitions),
            VersionId(1),
            VersionId(1),
            plan,
            plan.map(InvariantPlanContract::from_merged_plan),
            &InvariantPreparationControl::uninterrupted_proposal_validation(),
        )
        .unwrap()
    }
}
