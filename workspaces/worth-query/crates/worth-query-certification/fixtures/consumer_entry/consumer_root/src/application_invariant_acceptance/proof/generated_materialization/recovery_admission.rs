//! Refused advancement returns the original generated-output recovery.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryCancellationSource,
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
    },
    primary_graph::{
        WorthQueryGeneratedOutputRestorationRecovery,
        WorthQueryGeneratedOutputRestorationRecoveryStage,
        WorthQueryGeneratedOutputSuspensionRecovery,
        WorthQueryGeneratedOutputSuspensionRecoveryStage,
    },
};

pub(super) fn suspension(
    application: &ProgramApplication,
    recovery: WorthQueryGeneratedOutputSuspensionRecovery,
    scope: &WorthQueryRequestScope,
) -> WorthQueryGeneratedOutputSuspensionRecovery {
    refusals(application, recovery, scope, |recovery, scope| {
        application
            .continue_generated_output_suspension_recovery::<PlanarFinalOutputProducer<ConsumerSchema>>(
                recovery, scope,
            )
            .map_err(|failure| {
                let WorthQueryGeneratedOutputSuspensionRecoveryStage::ExecutionDenied(denial) =
                    failure.stage()
                else {
                    panic!("admission must refuse before inspecting suspension custody")
                };
                (denial, failure.into_recovery())
            })
    })
}

pub(super) fn restoration(
    application: &ProgramApplication,
    recovery: WorthQueryGeneratedOutputRestorationRecovery,
    scope: &WorthQueryRequestScope,
) -> WorthQueryGeneratedOutputRestorationRecovery {
    refusals(application, recovery, scope, |recovery, scope| {
        application
            .continue_generated_output_restoration_recovery(recovery, scope)
            .map_err(|failure| {
                let WorthQueryGeneratedOutputRestorationRecoveryStage::ExecutionDenied(denial) =
                    failure.stage()
                else {
                    panic!("admission must refuse before inspecting restoration custody")
                };
                (denial, failure.into_recovery())
            })
    })
}

fn refusals<Recovery, Output>(
    application: &ProgramApplication,
    mut recovery: Recovery,
    scope: &WorthQueryRequestScope,
    continue_recovery: impl Fn(
        Recovery,
        &WorthQueryRequestScope,
    ) -> Result<Output, (WorthQueryAdvancementDenial, Recovery)>,
) -> Recovery {
    let (denial, retained) = application
        .with_application_advancement(scope, |_phase| {
            continue_recovery(recovery, scope)
                .err()
                .expect("a nested opening must refuse recovery")
        })
        .expect("the outer installed advancement is admitted");
    assert_eq!(denial, WorthQueryAdvancementDenial::NestedOpening);
    recovery = retained;

    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    let expired = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        WorthQueryCancellationSource::new().token(),
    );
    for (scope, expected) in [
        (
            cancelled,
            WorthQueryManagedComputationInterruption::Cancelled,
        ),
        (
            expired,
            WorthQueryManagedComputationInterruption::DeadlineExceeded,
        ),
    ] {
        let (denial, retained) = continue_recovery(recovery, &scope)
            .err()
            .expect("interrupted admission must return recovery custody");
        assert_eq!(denial, WorthQueryAdvancementDenial::Interrupted(expected));
        recovery = retained;
    }
    for (memory, work) in [(1 << 20, 0), (0, 1_000)] {
        let budget = worth_foundational::ExecutionBudget::new(NonZeroUsize::MIN, memory, work);
        let (denial, retained) = with_recovery_request_budget(budget, || {
            continue_recovery(recovery, scope)
                .err()
                .expect("resource admission refuses before consuming recovery")
        });
        match denial {
            WorthQueryAdvancementDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ) if work == 0 => {}
            WorthQueryAdvancementDenial::Resource(
                WorthQueryManagedComputationResourceDenial::MemoryLimit {
                    requested,
                    admitted: 0,
                    level: WorthQueryMemoryLimitLevel::Policy,
                },
            ) if memory == 0 => assert!(requested > 0, "native request custody needs memory"),
            denial => panic!("the supplied resource limit must explain refusal: {denial:?}"),
        }
        recovery = retained;
    }
    recovery
}

fn with_recovery_request_budget<Output>(
    budget: worth_foundational::ExecutionBudget,
    work: impl FnOnce() -> Output,
) -> Output {
    use worth_query_host::facade::primary_graph::{
        bound_advancement_requests_on_this_thread_for_test,
        place_managed_computations_on_this_thread_for_test, WorthQueryExecutionObservation,
        WorthQueryExecutionPlacementForTest,
    };
    let _control_scope = WorthQueryExecutionObservation::install_on_this_thread();
    place_managed_computations_on_this_thread_for_test(WorthQueryExecutionPlacementForTest::Serial);
    bound_advancement_requests_on_this_thread_for_test(Some(budget));
    work()
}
