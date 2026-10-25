//! A genuine native DAG, using issued entity identities and ordinary transactions.
use super::super::validation_engine_fixtures::*;
use crate::mvcc::ValidatedRelationalProposal;
use crate::transactions::data::{
    AspectFieldPatch, BulkEntityCreateIntent, CommitResult, EntityReference, TransactionCommitError,
};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub(super) struct DagFixture {
    pub(super) runtime: RelationalRuntime,
    pub(super) nodes: Vec<crate::identity::data::EntityId>,
}

impl DagFixture {
    pub(super) fn new(count: usize) -> Self {
        Self::from_runtime(runtime_with_acyclicity_and_connectivity(), count)
    }

    pub(super) fn from_runtime(runtime: RelationalRuntime, count: usize) -> Self {
        let backing = crate::test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&backing);
        let keys: Vec<_> = (0..count)
            .map(|index| ClientKey::raw(format!("dag-node-{index}")))
            .collect();
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
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
                    .created_entity(&crate::transactions::data::CreatedEntityRef {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(3),
                        client_key,
                    })
                    .expect("actual issued entity")
            })
            .collect();
        crate::tests::support::release_test_commit_snapshot(&runtime, &receipt);
        Self { runtime, nodes }
    }

    pub(super) fn head(&self) -> crate::branch::RelationalBranchReferenceState {
        self.runtime
            .branch_reference_state(&crate::history::data::BranchId("main".into()))
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
        source: crate::identity::data::EntityId,
        target: crate::identity::data::EntityId,
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

    pub(super) fn validate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&self.runtime);
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.validate(execution, &self.runtime)
    }

    pub(super) fn merged_plan(&self, batch: WorkerIntentBatch) -> MergedCommitPlan {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&self.runtime);
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.merged_plan(
            &self.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .clone()
    }

    pub(super) fn publish(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Result<CommitResult, TransactionCommitError> {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&self.runtime);
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.commit(execution, &self.runtime)
    }
}

fn edge(
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    key: String,
) -> MutationIntent {
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
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(bytes), Some(work)),
            ),
            cancellation,
            deadline,
        })
        .unwrap()
}
