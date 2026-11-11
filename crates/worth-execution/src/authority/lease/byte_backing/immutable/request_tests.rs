use super::*;
use crate::{CancellationSource, ExecutionRequestArrayBuilder, SerialMemoryBudget, SerialRequest};

fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut charge = 0;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| charge = budget.charged_memory_bytes())
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    charge
}

#[test]
fn immutable_seal_retains_payload_and_actual_control_through_final_alias() {
    for payload in [b"".as_slice(), b"same original bytes".as_slice()] {
        let budget = SerialMemoryBudget::new(Some(1024 * 1024));
        let host =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let mut builder =
            ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&host), payload.len())
                .unwrap();
        builder.extend_from_slice(payload).unwrap();
        let bytes = builder.seal_immutable_bytes().unwrap();
        assert_eq!(bytes.bytes(), payload);
        assert_eq!(bytes.charged_payload_bytes(), Some(payload.len() as u64));
        let expected_control = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<ImmutableByteBacking>())
            .unwrap()
            .0
            .pad_to_align()
            .size() as u64;
        assert_eq!(
            budget.charged_memory_bytes(),
            payload.len() as u64 + expected_control
        );
        let alias = bytes.clone();
        drop(bytes);
        assert_eq!(
            budget.charged_memory_bytes(),
            payload.len() as u64 + expected_control
        );
        drop(alias);
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}

#[test]
fn immutable_seal_refuses_actual_control_and_refunds_existing_payload() {
    let payload = b"already admitted payload";
    let control = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<ImmutableByteBacking>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64;
    let budget = SerialMemoryBudget::new(Some(controller() + payload.len() as u64 + control - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&host), payload.len())
            .unwrap();
    builder.extend_from_slice(payload).unwrap();
    assert_eq!(budget.charged_memory_bytes(), payload.len() as u64);
    let failure = builder.seal_immutable_bytes();
    assert!(
        matches!(failure, Err(ExecutionRequestArrayDenial::Request(
        crate::WorkCeilingDenial::Admission(crate::LeaseDenial::MemoryExhausted(crate::MemoryLimitDenial::Capacity {
            requested, admitted, level: crate::MemoryLimitLevel::Policy { ancestor: 0 }
        }))
    )) if requested == control && admitted == control - 1),
        "{failure:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn immutable_seal_preserves_original_cancellation_and_incomplete_fill() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let mut builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&host), 3).unwrap();
    builder.extend_from_slice(b"abc").unwrap();
    cancellation.cancel();
    assert!(matches!(
        builder.seal_immutable_bytes(),
        Err(ExecutionRequestArrayDenial::Request(
            crate::WorkCeilingDenial::Stopped(crate::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let builder =
        ExecutionRequestArrayBuilder::allocate(ExecutionRequest::serial(&host), 3).unwrap();
    assert!(matches!(
        builder.seal_immutable_bytes(),
        Err(ExecutionRequestArrayDenial::Allocation(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
