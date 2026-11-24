//! A genuine native DAG built through owner-issued branch admission.
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionRequest,
    ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::branch::RelationalBranchReferenceState;
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::mvcc::{
    BranchBoundRelationalTransaction, RelationalTransactionIntent, ValidatedRelationalProposal,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
    ConnectivityMinimumEnforcement, DirectedTraversalKind, EntityKindRegistration,
    KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, BulkEntityCreateIntent, CommitResult, CreateIntent, CreatedEntityRef,
    EntityReference, MutationIntent, RelationSpec, TransactionCommitError, WorkerIntentBatch,
};

pub(super) use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

pub(in crate::relational::relational_certification) fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(8).expect("positive test worker count"),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .expect("native cycle court execution authority")
    })
}

fn begin_transaction(runtime: &RelationalRuntime) -> BranchBoundRelationalTransaction {
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("configured main branch remains owner-admissible");
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("main basis belongs to its issuing runtime")
}

pub(super) struct DagFixture {
    runtime: RelationalRuntime,
    pub(super) nodes: Vec<EntityId>,
}

impl DagFixture {
    pub(super) fn new(count: usize) -> Self {
        Self::from_runtime(
            RelationalRuntimeApi::builder()
                .schema_registry(acyclicity_and_connectivity_registry())
                .build()
                .expect("valid native runtime configuration"),
            count,
        )
    }

    pub(super) fn from_runtime(runtime: RelationalRuntime, count: usize) -> Self {
        let backing = serial_request();
        let execution = ExecutionRequest::serial(&backing);
        let keys: Vec<_> = (0..count)
            .map(|index| ClientKey::raw(format!("dag-node-{index}")))
            .collect();
        let mut txn = begin_transaction(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("dag-nodes").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(3),
                    client_keys: keys.clone(),
                    field_patches: vec![AspectFieldPatch::default(); count],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        let receipt = txn.commit(&execution, &runtime).unwrap();
        let nodes = keys
            .into_iter()
            .map(|client_key| {
                receipt
                    .created_entity(&CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(3),
                        client_key,
                    })
                    .expect("actual issued entity")
            })
            .collect();
        runtime
            .snapshots()
            .release_snapshot(&receipt.snapshot)
            .expect("release the node fixture's published snapshot");
        Self { runtime, nodes }
    }

    pub(super) fn head(&self) -> RelationalBranchReferenceState {
        self.runtime
            .branch_reference_state(&BranchId("main".into()))
            .unwrap()
    }

    pub(super) fn chain_batch(&self, forward_edges: usize) -> WorkerIntentBatch {
        let mut batch = WorkerIntentBatch::new("dag-chain");
        for (index, &source) in self.nodes.iter().enumerate() {
            for (offset, &target) in self
                .nodes
                .iter()
                .skip(index + 1)
                .take(forward_edges)
                .enumerate()
            {
                batch = batch.push(edge(source, target, format!("dag-edge-{index}-{offset}")));
            }
        }
        batch
    }

    pub(super) fn edge_batch(
        &self,
        source: EntityId,
        target: EntityId,
        key: &str,
    ) -> WorkerIntentBatch {
        WorkerIntentBatch::new(key).push(edge(source, target, key.to_owned()))
    }

    pub(super) fn graph_batch(&self, edges: &[(usize, usize)]) -> WorkerIntentBatch {
        let mut batch = WorkerIntentBatch::new("graph-oracle");
        for (index, &(source, target)) in edges.iter().enumerate() {
            batch = batch.push(edge(
                self.nodes[source],
                self.nodes[target],
                format!("oracle-{index}"),
            ));
        }
        batch
    }

    pub(super) fn release_snapshot(&self, receipt: &CommitResult) {
        self.runtime
            .snapshots()
            .release_snapshot(&receipt.snapshot)
            .unwrap();
    }

    pub(super) fn validate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let mut txn = begin_transaction(&self.runtime);
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.validate(execution, &self.runtime)
    }

    pub(super) fn publish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Result<CommitResult, TransactionCommitError> {
        let mut txn = begin_transaction(&self.runtime);
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.commit(execution, &self.runtime)
    }
}

fn edge(source: EntityId, target: EntityId, key: String) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw(key),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        fields: AspectFieldPatch::default(),
    }))
}

pub(super) fn request(
    bytes: u64,
    work: u64,
    cancellation: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(bytes), Some(work)),
            ),
            cancellation,
            deadline: deadline
                .map(|instant| worth_execution::MonotonicClock::system().deadline_at(instant)),
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
