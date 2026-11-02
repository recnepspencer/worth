//! Original-request initialized replacement keeps one exact physical allocation.
use std::cell::Cell;
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionPhysicalAllocationDenialKind,
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial, MapKernelStop,
    SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
};

struct Tracked<'a> {
    bit: usize,
    dropped: &'a Cell<usize>,
}
impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() | self.bit);
    }
}

#[test]
fn request_array_replacement_preserves_pointer_charge_and_initialized_prefix() {
    let cancel = CancellationSource::new();
    let quote = std::alloc::Layout::array::<Tracked<'_>>(2).unwrap().size() as u64;
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let dropped = Cell::new(0);
    let item = |bit| Tracked {
        bit,
        dropped: &dropped,
    };
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&request), 2).unwrap();
    builder.push(item(1)).unwrap();
    let pointer = builder.elements().as_ptr();
    builder.replace_initialized(0, item(2)).unwrap();
    assert_eq!(dropped.get(), 1);
    let error = builder.replace_initialized(1, item(4)).unwrap_err();
    let ExecutionRequestArrayDenial::Allocation(error) = error else {
        panic!("{error:?}")
    };
    assert_eq!(
        error.classify(),
        ExecutionAllocationDenialClass::Physical(
            ExecutionPhysicalAllocationDenialKind::UninitializedElement
        )
    );
    assert_eq!(error.requested_payload_bytes(), Some(quote));
    assert_eq!(dropped.get(), 1 | 4);
    assert_eq!((builder.len(), builder.element_count()), (1, 2));
    builder.push(item(8)).unwrap();
    let array = builder.seal().unwrap();
    assert_eq!(array.elements().as_ptr(), pointer);
    assert_eq!(array.charged_payload_bytes(), Some(quote));
    assert!(request.memory().unwrap().charged_memory_bytes() >= quote);
    assert_eq!((array[0].bit, array[1].bit), (2, 8));
    drop(array);
    assert_eq!(dropped.get(), 1 | 2 | 4 | 8);
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn request_array_replacement_preserves_original_cancellation_and_consumes_input() {
    let cancel = CancellationSource::new();
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let dropped = Cell::new(0);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&request), 1).unwrap();
    builder
        .push(Tracked {
            bit: 1,
            dropped: &dropped,
        })
        .unwrap();
    cancel.cancel();
    let error = builder
        .replace_initialized(
            0,
            Tracked {
                bit: 2,
                dropped: &dropped,
            },
        )
        .unwrap_err();
    assert_eq!(
        error,
        ExecutionRequestArrayDenial::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
    assert_eq!(dropped.get(), 2);
    assert_eq!(builder.elements()[0].bit, 1);
    assert!(builder.seal().is_err());
    assert_eq!(dropped.get(), 1 | 2);
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn request_array_replacement_checks_original_request_after_replaced_value_drop() {
    struct CancelOnDrop<'a>(Option<&'a CancellationSource>);
    impl Drop for CancelOnDrop<'_> {
        fn drop(&mut self) {
            if let Some(cancel) = self.0 {
                cancel.cancel();
            }
        }
    }
    let cancel = CancellationSource::new();
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&request), 1).unwrap();
    builder.push(CancelOnDrop(Some(&cancel))).unwrap();
    let error = builder
        .replace_initialized(0, CancelOnDrop(None))
        .unwrap_err();
    let expected = WorkCeilingDenial::Stopped(MapKernelStop::Cancelled);
    match error {
        ExecutionRequestArrayDenial::Request(cause) => assert_eq!(cause, expected),
        ExecutionRequestArrayDenial::Allocation(cause) => assert_eq!(
            cause.classify(),
            ExecutionAllocationDenialClass::Request(expected)
        ),
    }
    assert!(
        builder.elements()[0].0.is_none(),
        "replacement is initialized before old Drop"
    );
    assert!(builder.seal().is_err());
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}
