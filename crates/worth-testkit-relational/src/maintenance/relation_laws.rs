//! Ordinary SDK graph fixture shared by relation-law equality and scale courts.
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub fn request() -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1024 * 1024 * 1024),
                Some(1_000_000_000),
            ),
        ),
        CancellationSource::new().token(),
        None,
    )
}

pub struct World {
    pub runtime: RelationalRuntime,
    pub nodes: Vec<EntityId>,
    pub edges: Vec<(RelationId, usize, usize)>,
}
impl World {
    /// Each independent component has source -> transit -> target, plus a
    /// second target for fixed-size retargeting. Transit is not a source kind:
    /// deleting its outgoing edge must select its indirect source predecessor.
    pub fn new(execution: &ExecutionRequest<'_, '_>, components: usize) -> Self {
        Self::with_unrelated_kinds(execution, components, 0)
    }
    pub fn with_unrelated_kinds(
        execution: &ExecutionRequest<'_, '_>,
        components: usize,
        unrelated_kinds: u32,
    ) -> Self {
        Self::seed(execution, components, unrelated_kinds, false)
    }
    pub fn across_partitions(execution: &ExecutionRequest<'_, '_>, components: usize) -> Self {
        Self::seed(execution, components, 0, true)
    }
    pub fn empty_runtime(unrelated_kinds: u32) -> RelationalRuntime {
        let mut registry = RelationalSchemaRegistry::new();
        for kind in (1..=3).chain(100..100 + unrelated_kinds) {
            registry = registry
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(kind),
                    kind_name: format!("law.node.{kind}"),
                    schema_id: SchemaId("law".into()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: Default::default(),
                })
                .unwrap();
        }
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(9),
                kind_name: "law.reaches".into(),
                schema_id: SchemaId("law".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: Default::default(),
                relation_integrity: RelationIntegrityDeclarations::default()
                    .with_connectivity_minimum_contracts(vec![
                        ConnectivityMinimumContractDeclaration {
                            contract_id: "source_reaches_target".into(),
                            source_kind_ids: vec![KindId(1)],
                            target_kind_ids: vec![KindId(3)],
                            minimum_reachable_targets: 1,
                            enforcement_boundary:
                                ConnectivityMinimumEnforcement::SnapshotPublication,
                        },
                    ]),
            })
            .unwrap();
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(10),
                kind_name: "law.unrelated".into(),
                schema_id: SchemaId("law".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: Default::default(),
                relation_integrity: Default::default(),
            })
            .unwrap();
        RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build()
            .unwrap()
    }
    fn seed(
        execution: &ExecutionRequest<'_, '_>,
        components: usize,
        unrelated_kinds: u32,
        split_path: bool,
    ) -> Self {
        let runtime = Self::empty_runtime(unrelated_kinds);
        let mut batch = WorkerIntentBatch::new("independent-law-components");
        let mut keys = Vec::new();
        let mut edge_keys = Vec::new();
        for component in 0..components {
            for offset in 0..4 {
                let kind_id = KindId(match offset {
                    0 => 1,
                    1 => 2,
                    _ => 3,
                });
                let key = CreatedEntityRef {
                    partition_id: if split_path {
                        PartitionId(offset + 1)
                    } else {
                        PartitionId::main()
                    },
                    kind_id,
                    client_key: ClientKey::raw(format!("node-{component}-{offset}")),
                };
                batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: key.partition_id,
                    kind_id,
                    client_key: key.client_key.clone(),
                    fields: Default::default(),
                })));
                keys.push(key);
            }
            for offset in 0..2 {
                let source = component * 4 + offset;
                let target = source + 1;
                let key = CreatedRelationRef {
                    partition_id: keys[source].partition_id,
                    kind_id: KindId(9),
                    client_key: ClientKey::raw(format!("edge-{component}-{offset}")),
                    source: EntityReference::Created(keys[source].clone()),
                    target: EntityReference::Created(keys[target].clone()),
                };
                batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: key.partition_id,
                        kind_id: key.kind_id,
                        client_key: key.client_key.clone(),
                        source: key.source.clone(),
                        target: key.target.clone(),
                        fields: Default::default(),
                    },
                )));
                edge_keys.push((key, source, target));
            }
        }
        let mut world = Self {
            runtime,
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        let seeded = world.commit(execution, batch).unwrap();
        world.nodes = keys
            .iter()
            .map(|key| seeded.created_entity(key).unwrap())
            .collect();
        world.edges = edge_keys
            .iter()
            .map(|(key, source, target)| (seeded.created_relation(key).unwrap(), *source, *target))
            .collect();
        world.release(&seeded);
        world
    }
    pub fn add_hub_fanout(&self, execution: &ExecutionRequest<'_, '_>, components: usize) {
        let mut batch = WorkerIntentBatch::new("unrelated-kind-at-edited-source");
        for component in 0..components {
            batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: self.nodes[1].partition_id,
                    kind_id: KindId(10),
                    client_key: ClientKey::raw(format!("unrelated-edge-{component}")),
                    source: EntityReference::Existing(self.nodes[1]),
                    target: EntityReference::Existing(self.nodes[component * 4 + 2]),
                    fields: Default::default(),
                },
            )));
        }
        let committed = self.commit(execution, batch).unwrap();
        self.release(&committed);
    }
    pub fn commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Result<CommitResult, TransactionCommitError> {
        let (_, basis) = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap();
        let mut tx = self
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
        tx.commit(execution, &self.runtime)
    }
    pub fn release(&self, result: &CommitResult) {
        self.runtime
            .snapshots()
            .release_snapshot(&result.snapshot)
            .unwrap();
    }
    pub fn insert(&self, source: usize, target: usize, key: &str) -> MutationIntent {
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(9),
            client_key: ClientKey::raw(key),
            source: EntityReference::Existing(self.nodes[source]),
            target: EntityReference::Existing(self.nodes[target]),
            fields: Default::default(),
        }))
    }
    pub fn retarget(&self, edge: usize, target: usize) -> MutationIntent {
        let (relation_id, source, _) = self.edges[edge];
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id,
                kind_id: KindId(9),
                source: EntityReference::Existing(self.nodes[source]),
                target: EntityReference::Existing(self.nodes[target]),
            },
        ))
    }
    pub fn delete(&self, edge: usize) -> MutationIntent {
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id: self.edges[edge].0,
        }))
    }
}
