use super::*;
use crate::domain_computation::primary_graph::value_decode::{
    ValueDecodeSession, WorthQueryValueDecodeDenial as Decode,
};
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{CancellationSource, SerialRequest};
use worth_foundational::facade::AspectValue;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Domain;
impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provider domain refusal")
    }
}
impl std::error::Error for Domain {}

fn backing(cancel: &CancellationSource) -> SerialRequest {
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
fn reject_legacy_codec(session: &ValueDecodeSession<'_>) {
    // The real strict managed path rejects a legacy scalar codec before it
    // can decode without admission. No installed row or permission is forged.
    assert!(session
        .decode::<U64ApplicationValueBinding>("Source", "Value", "Number", &AspectValue::UInt64(1))
        .is_none());
    assert_eq!(session.denial(), Some(Decode::BindingContractMismatch));
}
fn controlled_cause(result: Result<(), WorthQueryManagedIndexWorkDenial>) -> Stop {
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
