//! Actual performed reversal evidence under the original finite memory owner.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::correction_source;
use std::{num::NonZeroUsize, sync::Arc};
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, LeaseDenial, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

#[test]
fn performed_correction_capture_preserves_original_payer_and_preclone_refusal() {
    let world = correction_source::install();
    let source = correction_source::edit(&world, "funded-witness-source", "edited-label");
    let prepared = correction_source::prepare(&world, &source, "funded-witness-reverse");
    let reversed = correction_source::committed(correction_source::publish(&world, prepared));
    assert!(reversed.aftermath_causality().is_some());
    let receipt = reversed.commit_reference();
    let native = SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            NonZeroUsize::MIN,
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&native);
    assert!(!execution.has_accounting());
    for saturate in [false, true] {
        let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
        let parent =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let mut retained = None;
        let mut reached_copy_refusal = false;
        let handoff = ExecutionRequest::serial(&parent).run_native_with_checkpoint(|context| {
            assert!(context.has_accounting());
            let obstruction = saturate.then(|| {
                let capture_frame =
                    ExecutionWorkCeiling::scope_memory_bound_for_test(None, Some(&native)).unwrap();
                budget
                    .reserve(
                        budget.limit().unwrap() - budget.charged_memory_bytes() - capture_frame,
                    )
                    .unwrap()
            });
            let captured = world
                .application
                .primary_provider
                .capture_correction_witness(execution, receipt);
            if saturate {
                match captured {
                    Err(CorrectionWitnessDenial::CopyAdmission(WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(_),
                    ))) => reached_copy_refusal = true,
                    Err(cause) => panic!("expected copy admission refusal, got {cause:?}"),
                    Ok(None) => panic!("expected copy admission refusal, got no witness"),
                    Ok(Some(_)) => panic!("expected copy admission refusal, got a witness"),
                }
            } else {
                retained = captured.unwrap();
            }
            drop(obstruction);
        });
        if saturate {
            assert!(reached_copy_refusal);
            assert!(
                handoff.is_err(),
                "the original owner preserves its actual stop"
            );
            assert!(retained.is_none());
            assert_eq!(budget.charged_memory_bytes(), 0);
        } else {
            handoff.unwrap();
            let witness = retained.unwrap();
            let sealed = witness.aftermath.as_ref().unwrap();
            assert_eq!(Some(sealed), reversed.aftermath_causality());
            let copied_backing = sealed.owned_receipt_backing_bytes().unwrap();
            assert!(
                copied_backing > 0,
                "this is a real performed reversal, with copied receipts/root"
            );
            let held = budget.charged_memory_bytes();
            assert!(
                held > copied_backing,
                "the same original payer also retains the witness Arc"
            );
            assert_eq!(held, witness._memory.as_ref().unwrap().bytes());
            let sibling = Arc::clone(&witness);
            drop(witness);
            assert_eq!(budget.charged_memory_bytes(), held);
            drop(sibling);
            assert_eq!(budget.charged_memory_bytes(), 0);
        }
    }
    let retry = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(8 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let witness = world
        .application
        .primary_provider
        .capture_correction_witness(ExecutionRequest::serial(&retry), receipt)
        .unwrap()
        .unwrap();
    assert_eq!(witness.aftermath.as_ref(), reversed.aftermath_causality());
    drop(witness);
    assert_eq!(retry.memory().unwrap().charged_memory_bytes(), 0);
}
