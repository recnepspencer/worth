//! Live payloads expose premature release and double charging across capture stages.
use super::*;
use std::cell::Cell;
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};

#[test]
fn original_retained_capture_preserves_live_backing_through_success_and_refusal() {
    let elements = 8;
    let quote = std::alloc::Layout::array::<u64>(elements).unwrap().size() as u64;
    let frame_request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let frame = ExecutionWorkCeiling::scope_memory_bound_for_test(None, Some(&frame_request))
        .expect("the actual Serial unit native frame has a checked quote");
    for permit_second in [true, false] {
        let budget =
            SerialMemoryBudget::new(Some(frame + quote * if permit_second { 3 } else { 2 }));
        let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
        let execution = ExecutionRequest::serial(&serial);
        let mut memory = Some(budget.reserve(quote).unwrap());
        let original = vec![7_u64; elements];
        let first = run_retained_stage(&execution, &mut memory, |context, memory| {
            checkpoint(context, elements as u64).map_err(MapKernelFailure::Stop)?;
            memory
                .expect("actual original Serial payer")
                .resize(quote)
                .map_err(MapKernelFailure::Domain)?;
            Ok(original.clone())
        })
        .unwrap_or_else(|_| panic!("first admitted copy must reach capture"));
        assert!(budget.charged_memory_bytes() >= quote * 2);
        let entered = Cell::new(false);
        let copied = Cell::new(false);
        let second = run_retained_stage(&execution, &mut memory, |context, memory| {
            entered.set(true);
            checkpoint(context, elements as u64).map_err(MapKernelFailure::Stop)?;
            memory
                .expect("same actual original Serial payer")
                .resize(quote)
                .map_err(MapKernelFailure::Domain)?;
            copied.set(true);
            Ok(first.clone())
        });
        assert!(
            entered.get(),
            "a frame denial cannot substitute for capture admission"
        );
        let second = if permit_second {
            let value = second.unwrap_or_else(|_| panic!("lawful combined backing is funded"));
            assert!(copied.get());
            assert_eq!(value, original);
            assert!(budget.charged_memory_bytes() >= quote * 3);
            Some(value)
        } else {
            assert!(
                matches!(
                    second,
                    Err(CaptureStop::Kernel(ScopeStop::Failure(
                        MapKernelFailure::Domain(RetainedStorageDenial::Memory(_))
                    )))
                ),
                "the actual second-copy memory refusal survives the original context"
            );
            assert!(!copied.get(), "refusal precedes the second copy");
            assert!(budget.charged_memory_bytes() >= quote * 2);
            None
        };
        assert_eq!(
            original, first,
            "the original and prior capture stay live after either result"
        );
        drop(second);
        drop(first);
        drop(original);
        drop(memory);
        assert_eq!(
            budget.charged_memory_bytes(),
            0,
            "final-owner release returns all backing"
        );
    }
}
