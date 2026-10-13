//! Interrupted adoption and delayed recovery release preserve one publication.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryGeneratedOutputRestorationRecovery,
    WorthQueryGeneratedOutputRestorationRecoveryStage, WorthQueryGeneratedOutputSuspensionRecovery,
    WorthQueryGeneratedOutputSuspensionRecoveryStage, WorthQueryRestoredGeneratedOutput,
    WorthQuerySuspendedGeneratedOutput,
};

pub(super) fn suspension(
    application: &ProgramApplication,
    recovery: WorthQueryGeneratedOutputSuspensionRecovery,
    scope: &WorthQueryRequestScope,
) -> WorthQuerySuspendedGeneratedOutput {
    let branch = recovery.product_branch();
    publication_stops(
        application,
        recovery,
        scope,
        branch,
        WorthQueryGeneratedOutputSuspensionRecoveryStage::RecoveryHandoff,
        WorthQueryGeneratedOutputSuspensionRecoveryStage::AdoptionPublication,
        |recovery, scope| {
            application
                .continue_generated_output_suspension_recovery::<PlanarFinalOutputProducer<ConsumerSchema>>(recovery, scope)
                .map_err(|failure| (failure.stage(), failure.into_recovery()))
        },
    )
}

pub(super) fn restoration(
    application: &ProgramApplication,
    recovery: WorthQueryGeneratedOutputRestorationRecovery,
    scope: &WorthQueryRequestScope,
) -> WorthQueryRestoredGeneratedOutput {
    let branch = recovery.product_branch();
    publication_stops(
        application,
        recovery,
        scope,
        branch,
        WorthQueryGeneratedOutputRestorationRecoveryStage::RecoveryHandoff,
        WorthQueryGeneratedOutputRestorationRecoveryStage::AdoptionPublication,
        |recovery, scope| {
            application
                .continue_generated_output_restoration_recovery(recovery, scope)
                .map_err(|failure| (failure.stage(), failure.into_recovery()))
        },
    )
}

fn publication_stops<Recovery, Output, Stage: Copy + Eq + std::fmt::Debug>(
    application: &ProgramApplication,
    recovery: Recovery,
    scope: &WorthQueryRequestScope,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    handoff: Stage,
    publication: Stage,
    continue_recovery: impl Fn(Recovery, &WorthQueryRequestScope) -> Result<Output, (Stage, Recovery)>,
) -> Output {
    let current_commit = || {
        application
            .with_host_advancement(|phase| {
                application
                    .on_branch(branch)
                    .select(&phase)
                    .expect("the recovery branch remains selectable")
                    .product()
                    .selected_commit()
                    .clone()
            })
            .expect("the installed host opens its observation")
    };
    let before = current_commit();
    let recovery = retained_at(
        handoff,
        interrupt_recovery_adoption(application, |interrupted_scope| {
            with_recovery_cleanup_pressure(application, || {
                continue_recovery(recovery, interrupted_scope)
            })
        }),
    );
    assert_eq!(
        current_commit(),
        before,
        "interrupted adoption cannot move the product head"
    );
    let recovery = retained_at(
        handoff,
        with_recovery_cleanup_pressure(application, || continue_recovery(recovery, scope)),
    );
    let recovery = retained_at(publication, continue_recovery(recovery, scope));
    assert_eq!(
        current_commit(),
        before,
        "handoff release does not publish a product"
    );
    let recovery = retained_at(
        handoff,
        with_recovery_cleanup_pressure(application, || continue_recovery(recovery, scope)),
    );
    let published = current_commit();
    assert_ne!(
        published, before,
        "settled adoption must publish the product"
    );
    let recovery = retained_at(
        handoff,
        with_recovery_cleanup_pressure(application, || continue_recovery(recovery, scope)),
    );
    assert_eq!(
        current_commit(),
        published,
        "a published retry must only release custody"
    );
    let result = continue_recovery(recovery, scope)
        .unwrap_or_else(|(stage, _)| panic!("fresh retry completes retained recovery: {stage:?}"));
    assert_eq!(
        current_commit(),
        published,
        "finishing recovery cannot duplicate its commit"
    );
    result
}

fn retained_at<Recovery, Output, Stage: Eq + std::fmt::Debug>(
    expected: Stage,
    result: Result<Output, (Stage, Recovery)>,
) -> Recovery {
    let (stage, recovery) = result
        .err()
        .expect("the owned interruption must retain recovery");
    assert_eq!(stage, expected);
    recovery
}

fn with_recovery_cleanup_pressure<Output>(
    application: &ProgramApplication,
    work: impl FnOnce() -> Output,
) -> Output {
    application.with_owner_cleanup_capacity_exhausted_for_test(work)
}

fn interrupt_recovery_adoption<Output>(
    application: &ProgramApplication,
    work: impl FnOnce(&WorthQueryRequestScope) -> Output,
) -> Output {
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let pause = application
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    std::thread::scope(|threads| {
        let controller = threads.spawn(|| {
            assert!(
                pause.wait_until_reached(std::time::Duration::from_secs(10)),
                "adoption must reach the real product comparison"
            );
            cancellation.cancel();
            pause.release();
        });
        let result = work(&scope);
        controller
            .join()
            .expect("the interruption controller completes");
        result
    })
}
