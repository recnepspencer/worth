use worth_execution::ExecutionRequest;
use worth_execution::MapKernelStop;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexProviderStop as ProviderStop;
use std::cell::RefCell;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexStop as Stop;
use worth_execution::WorkCeilingDenial;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexWorkDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::invocation::controlled;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::invocation::decoded_provider;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::invocation::provider_result;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::value_decode::session::ValueDecodeSession;
use worth_query_execution::facade::domain_computation::WorthQueryValueDecodeDenial as Decode;
use std::cell::Cell;
use std::num::NonZeroUsize;
use worth_execution::CancellationSource;
use worth_execution::SerialRequest;
use worth_foundational::facade::AspectValue;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Domain;
impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provider domain refusal")
    }
}
impl std::error::Error for Domain {}

pub fn backing(cancel: &CancellationSource) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        cancel.token(),
        None,
    )
}
pub fn reject_legacy_codec(session: &ValueDecodeSession<'_>) {
    // The real strict managed path rejects a legacy scalar codec before it
    // can decode without admission. No installed row or permission is forged.
    assert!(session
        .decode::<U64ApplicationValueBinding>("Source", "Value", "Number", &AspectValue::UInt64(1))
        .is_none());
    assert_eq!(
        session.denial_for_test(),
        Some(Decode::BindingContractMismatch)
    );
}
pub fn controlled_cause(result: Result<(), WorthQueryManagedIndexWorkDenial>) -> Stop {
    match result {
        Err(WorthQueryManagedIndexWorkDenial::Controlled(cause)) => cause,
        other => panic!("expected actual controlled refusal: {other:?}"),
    }
}

#[test]
fn managed_decoder_preserves_first_refusal_and_distinct_outer_request_stop() {
    let catalog = Default::default();
    let cancel = CancellationSource::new();
    let storage = backing(&cancel);
    let execution = ExecutionRequest::serial(&storage);
    let entered = Cell::new(false);
    let first = Stop::Request(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling));
    let result = decoded_provider(&execution, &catalog, |control, session| {
        entered.set(true);
        assert_eq!(control.checkpoint(1_000_001), Err(first.clone()));
        reject_legacy_codec(session);
        Ok(()) // A provider can catch both; the actual first stop survives.
    });
    assert!(entered.get());
    assert_eq!(controlled_cause(result), first);

    for return_domain in [false, true] {
        let cancel = CancellationSource::new();
        let storage = backing(&cancel);
        let execution = ExecutionRequest::serial(&storage);
        let result = decoded_provider(&execution, &catalog, |control, session| {
            reject_legacy_codec(session);
            assert_eq!(
                control.checkpoint(1),
                Err(Stop::ValueDecode(Decode::BindingContractMismatch))
            );
            if return_domain {
                provider_result::<(), Domain>(
                    &execution,
                    control,
                    Err(ProviderStop::Domain(Domain)),
                )
            } else {
                Ok(())
            }
        });
        assert_eq!(
            controlled_cause(result),
            Stop::ValueDecode(Decode::BindingContractMismatch)
        );
    }

    let cancel = CancellationSource::new();
    let storage = backing(&cancel);
    let execution = ExecutionRequest::serial(&storage);
    let result = decoded_provider(&execution, &catalog, |_, session| {
        reject_legacy_codec(session);
        cancel.cancel();
        Ok(())
    });
    assert_eq!(
        controlled_cause(result),
        Stop::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
}

#[test]
fn original_content_reservation_keeps_its_payer_and_first_memory_or_cancel_refusal() {
    use worth_execution::LeaseDenial;
    use worth_execution::SerialMemoryBudget;
    let cancel = CancellationSource::new();
    // The finite original ceiling funds framework entry plus the requested hold.
    let ceiling = 4096;
    let memory = SerialMemoryBudget::new(Some(ceiling));
    let storage = SerialRequest::from_memory(memory.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&storage);
    let held = controlled(&execution, |control| {
        Ok(control.reserve_content_memory_for_test(64)?)
    })
    .unwrap()
    .expect("actual accounted original reservation");
    assert_eq!(memory.charged_memory_bytes(), 64);
    drop(held);
    assert_eq!(memory.charged_memory_bytes(), 0);

    let expected = RefCell::new(None);
    let result = controlled(&execution, |control| {
        let first = control
            .reserve_content_memory_for_test(ceiling + 1)
            .unwrap_err();
        assert!(matches!(
            first,
            Stop::Request(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                _
            )))
        ));
        assert_eq!(
            control.reserve_content_memory_for_test(1).unwrap_err(),
            first
        );
        *expected.borrow_mut() = Some(first);
        Ok(()) // original memory refusal cannot be swallowed by the caller
    });
    assert_eq!(controlled_cause(result), expected.into_inner().unwrap());
    assert_eq!(memory.charged_memory_bytes(), 0);

    let cancel = CancellationSource::new();
    let storage = backing(&cancel);
    let execution = ExecutionRequest::serial(&storage);
    let result = controlled(&execution, |control| {
        cancel.cancel();
        assert!(control.reserve_content_memory_for_test(1).is_err());
        Ok(())
    });
    assert_eq!(
        controlled_cause(result),
        Stop::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    );
}

/// Reuse this original-controls fixture without creating another source-court root.
pub fn with_original_content_scope<R>(
    body: impl FnOnce(
        &ExecutionRequest<'_, '_>,
        &worth_execution::SerialMemoryBudget,
        &CancellationSource,
    ) -> R,
) -> R {
    let cancel = CancellationSource::new();
    let memory = worth_execution::SerialMemoryBudget::new(Some(1_000_000));
    let backing = SerialRequest::from_memory(memory.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&backing);
    body(&execution, &memory, &cancel)
}
