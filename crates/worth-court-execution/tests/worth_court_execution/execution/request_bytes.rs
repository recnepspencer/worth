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
