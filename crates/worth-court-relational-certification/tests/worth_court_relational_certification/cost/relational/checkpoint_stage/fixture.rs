//! Real Native population and independent MessagePack observation for the courts.
use super::*;
pub(super) fn runtime_with_rows(rows: usize) -> RelationalRuntime {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let runtime = AspectSchemaFixture::default().build_runtime();
    for ordinal in 0..rows {
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("retained-publication").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw(format!("record-{ordinal}")),
                        fields: Default::default(),
                    }),
                )),
                Allocation::SystemAllocation,
            )
            .unwrap();
        transaction.commit(&execution, &runtime).unwrap();
    }
    runtime
}

/// The checkpoint length or its refusal; every retained byte is released.
pub(super) fn memory_outcome(leased: bool, memory: u64, cancelled: bool) -> Result<usize, Denial> {
    let source = CancellationSource::new();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory), None),
    );
    let serial = SerialRequest::from_policy(&policy, source.token(), None);
    let lease = leased.then(|| {
        crate::leased_preparation::ordinary_context::authority()
            .request_lease(worth_execution::LeaseRequest {
                policy,
                cancellation: source.token(),
                deadline: None,
            })
            .unwrap()
    });
    let request = match &lease {
        Some(lease) => ExecutionRequest::leased(lease),
        None => ExecutionRequest::serial(&serial),
    };
    let runtime = runtime_with_rows(0);
    if cancelled {
        source.cancel();
    }
    let outcome = runtime
        .durability_authority()
        .native_checkpoint(&request, request.allocation_policy())
        .map(|checkpoint| checkpoint.bytes().len());
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    if let Some(lease) = &lease {
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
    outcome
}
