use crate::facade::*;
use crate::tests::support::{evaluate_on_demand, version_ab};

pub(crate) type AsyncNodeTestRuntime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn async_node_capability_declaration(node: NodeId) -> AsyncNodeCapabilityDeclaration {
    AsyncNodeCapabilityDeclaration::new(
        node,
        AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(7))
            .with_max_payload_bytes(1024),
    )
}

pub(crate) fn async_node_capability_with_dependents(
    node: NodeId,
    dependents: impl IntoIterator<Item = NodeId>,
) -> AsyncNodeCapabilityDeclaration {
    async_node_capability_declaration(node).with_declared_dependent_cancellation_nodes(dependents)
}

pub(crate) fn settle_async_dependency_baseline(
    graph: &mut SignalGraph,
    nodes: impl IntoIterator<Item = NodeId>,
) {
    let mut evaluator = |_id: NodeId, _graph: &SignalGraph| Ok(version_ab(1, 0));
    for node in nodes {
        evaluate_on_demand(&mut *graph, node, &mut evaluator)
            .expect("async dependency baseline should settle before lifecycle admission");
    }
}

pub(crate) fn raw_async_node_completion(
    handle: ResourceRequestHandle,
    attempt: ResourceAttemptId,
    payload_contract_digest: ResourcePayloadContractDigest,
    payload_byte_len: u64,
) -> RawCompletionEnvelope {
    RawCompletionEnvelope::new(
        handle.request_id(),
        handle.generation(),
        handle.branch_epoch(),
        attempt,
        payload_contract_digest,
        payload_byte_len,
    )
}

pub(crate) fn admit_and_commit_async_node_completion(
    runtime: &mut AsyncNodeTestRuntime,
    handle: ResourceRequestHandle,
    attempt: ResourceAttemptId,
    payload_contract_digest: ResourcePayloadContractDigest,
    payload_byte_len: u64,
) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let admitted_completion = runtime
        .admit_resource_completion(raw_async_node_completion(
            handle,
            attempt,
            payload_contract_digest,
            payload_byte_len,
        ))
        .admitted_completion()
        .expect("completion should admit");
    let mut ctx = ();
    runtime
        .transaction(request_execution, &mut ctx, |tx| {
            let staged = tx.stage_admitted_resource_completion(admitted_completion)?;
            tx.commit_staged_resource_completion(staged.staged_effect())?;
            Ok(())
        })
        .expect("completion should commit");
}
