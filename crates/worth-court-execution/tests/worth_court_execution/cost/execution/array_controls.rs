use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder,
    ExecutionRequestArrayDenial, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
#[test]
fn large_inline_values_check_controls_before_the_next_indivisible_move() {
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&original), 2).unwrap();
    builder.push([0u8; 128 * 1024]).unwrap();
    cancellation.cancel();
    assert!(matches!(
        builder.push([1u8; 128 * 1024]),
        Err(ExecutionRequestArrayDenial::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        builder.len(),
        1,
        "the stopped large move never entered initialized storage"
    );
}
