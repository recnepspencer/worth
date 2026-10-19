use super::{validate_admitted, QueryRequestExecution, WorthQueryAdvancementDenial};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial;
use std::{cell::Cell, num::NonZeroUsize};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::ApplicationNativeInputDecodeResources;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn decoder_is_not_called_when_its_declared_work_or_memory_is_refused() {
    for (memory, work, quoted, expected_work_refusal) in [
        (
            1_024,
            2,
            ApplicationNativeInputDecodeResources::new(3, 1),
            true,
        ),
        (
            1_024,
            10,
            ApplicationNativeInputDecodeResources::new(1, 2_048),
            false,
        ),
    ] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, memory, Some(work)),
        );
        let execution =
            QueryRequestExecution::open(RuntimeWorldExecutionPlacement::Serial(policy), &scope);
        let calls = Cell::new(0);
        let entered = Cell::new(false);
        let refusal = execution
            .decision_stage(|context| {
                entered.set(true);
                validate_admitted(&execution, context, quoted, || {
                    calls.set(calls.get() + 1);
                    true
                })
            })
            .and_then(|result| result)
            .unwrap_err();
        assert!(
            entered.get(),
            "the native decoder-admission funnel was reached"
        );
        assert_eq!(calls.get(), 0);
        if expected_work_refusal {
            assert!(matches!(
                refusal,
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted
                )
            ));
        } else {
            assert!(matches!(
                refusal,
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::MemoryLimit { .. }
                )
            ));
        }
    }
}
