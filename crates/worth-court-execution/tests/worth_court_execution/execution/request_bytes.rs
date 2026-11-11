use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

#[test]
fn immutable_byte_conversion_retains_the_same_allocation_and_original_memory_charge() {
    // Kernel entry also accounts for temporary SDK state. This case disputes
    // retained payload custody, not the separate entry-admission ceiling.
    let memory = SerialMemoryBudget::new(None);
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&request), 4).unwrap();
    builder.extend_from_slice(&[1, 2, 3, 4]).unwrap();
    let array = builder.seal().unwrap();
    let address = array.elements().as_ptr();
    let bytes = array.into_immutable_bytes();
    assert_eq!(bytes.bytes().as_ptr(), address);
    assert_eq!(bytes.charged_payload_bytes(), Some(4));
    let retained = bytes.clone();
    drop(bytes);
    assert_eq!(memory.charged_memory_bytes(), 4);
    assert_eq!(retained.bytes(), &[1, 2, 3, 4]);
    drop(retained);
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn byte_copy_refuses_original_cancellation_without_sealing_a_partial_payload() {
    let cancellation = CancellationSource::new();
    let request =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&request), 4).unwrap();
    builder.extend_from_slice(&[1, 2]).unwrap();
    cancellation.cancel();
    assert!(matches!(
        builder.extend_from_slice(&[3, 4]),
        Err(ExecutionRequestArrayDenial::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(builder.elements(), &[1, 2]);
    assert!(matches!(
        builder.seal(),
        Err(ExecutionRequestArrayDenial::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        ))
    ));
}

#[test]
fn staged_byte_seal_refuses_incomplete_fill_and_refunds_the_original_payload() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let (buffer, _) = request
        .run_native_with_checkpoint(|context| {
            worth_execution::ExecutionByteBuffer::allocate_in_stage(2, request, context)
        })
        .unwrap();
    let mut buffer = buffer.unwrap();
    assert_eq!(budget.charged_memory_bytes(), 2);
    buffer.extend_from_slice(&[1]).unwrap();
    let refused = buffer.seal().unwrap_err();
    assert_eq!(
        refused.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::IncompleteSeal
        )
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn staged_worker_only_bytes_keep_cancellation_without_a_memory_ticket() {
    let cancellation = CancellationSource::new();
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ),
        deadline: None,
        cancellation: cancellation.token(),
    })
    .unwrap();
    let request = ExecutionRequest::leased(&lease);
    let (buffer, _) = request
        .run_native_with_checkpoint(|context| {
            assert!(!context.has_accounting());
            worth_execution::ExecutionByteBuffer::allocate_in_stage(2, request, context)
        })
        .unwrap();
    let mut buffer = buffer.unwrap();
    assert_eq!(lease.charged_memory_bytes(), 0);
    buffer.extend_from_slice(&[1, 2]).unwrap();
    cancellation.cancel();
    assert_eq!(
        buffer.seal().unwrap_err().classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        )
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}
