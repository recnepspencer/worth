//! A genuine native DAG, using issued entity identities and ordinary transactions.
use super::super::validation_engine_fixtures::*;
use super::packet_capacity::edge;
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::mvcc::ValidatedRelationalProposal;
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::intents::mutation_intent::BulkEntityCreateIntent;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::CommitResult;
use worth_execution::ExecutionRequest;

pub(super) struct DagFixture {
    pub(super) runtime: RelationalRuntime,
    pub(super) nodes: Vec<crate::identity::data::EntityId>,
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
        let backing = worth_harness::facade::execution_fixture::serial_request();
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
                    .created_entity(&crate::transactions::data::primitives::CreatedEntityRef {
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
