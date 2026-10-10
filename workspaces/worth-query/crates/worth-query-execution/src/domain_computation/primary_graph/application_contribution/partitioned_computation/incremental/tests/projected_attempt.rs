//! An admitted computation and the projection custody it can seal.
use super::*;

/// [`attempt_in`] with the request's execution opened under `placement`.
pub(super) fn attempt_placed<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    attempt_placed_observing(world, installed, prior, request, placement, |_| {})
}

/// The same fixture, observed immediately before computation reserves memory.
pub(super) fn attempt_placed_observing<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    mut before_computation: impl FnMut(&QueryRequestExecution<'_>),
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    let _placement =
        super::super::super::super::request_execution::FixturePlacement::enter(placement);
    let mut observed = None;
    let finished = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request();
            let principal =
                authenticated_principal(&(phase_custody).test_host_request(), world, request);
            let account =
                resolved_account(&(phase_custody).test_host_request(), world, "open", request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&(phase_custody).test_host_request())
                .authorize_operation(
                    &phase_custody.phase_for_application(&world.application),
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    request,
                )
                .unwrap();
            installed.owner.gathered().lock().unwrap().clear();
            runs();
            tree_runs();
            ComputationPrior::hand_in_test(prior);
            SealedComputationRun::keep_in_test(None);
            let mut outcome = None;
            let completed = world.invariant.project_admitted_operation(
                &phase_custody.phase_for_application(&world.application),
                &admission,
                |reader, root| {
                    outcome = Some((|| -> Outcome {
                        let execution = phase
                            .execution_for(&world.application.product_runtime)
                            .expect("the fixture phase belongs to its installed world");
                        before_computation(execution);
                        let computed = installed
                            .prepare_through(reader, execution, root)?
                            .compute(WorthQueryManagedComputationExecution::new(execution))?;
                        let charged = computed.charged_work();
                        Ok((computed.complete()?, charged))
                    })());
                },
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            );
            let (projection, work) = match completed {
                Ok(completed) => {
                    let (_, projection, work) = completed.into_parts();
                    (Some(projection), work)
                }
                Err(denial) => {
                    let interruption = request
                        .interruption()
                        .expect("only interrupted custody is refused");
                    assert_eq!(
                        denial
                            .invariant_denial()
                            .and_then(|denial| denial.source_retention_interruption()),
                        Some(interruption)
                    );
                    (
                        None,
                        denial.projection_work().expect("the reader performed work"),
                    )
                }
            };
            let outcome = outcome.expect("the admitted computation ran");
            ComputationPrior::hand_in_test(None);
            // An interrupted request retains no sealed projection or computation run.
            let sealed = if request.interruption().is_some() {
                Err(())
            } else {
                world
                    .application
                    .begin_projected_application_read_attempt(
                        &phase_custody.phase_for_application(&world.application),
                        admission,
                        projection.expect("an uninterrupted projection sealed"),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .complete_projected_dependencies(
                        &phase_custody.phase_for_application(&world.application),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .map(|_| SealedComputationRun::kept_in_test())
                    .map_err(|_| ())
            };
            let mut gathered = installed.owner.gathered().lock().unwrap().clone();
            gathered.sort_unstable();
            observed = Some(Attempt {
                outcome,
                work,
                runs: runs(),
                tree_runs: tree_runs(),
                gathered,
                sealed,
            });
        });
    let attempt = observed.expect("the admitted computation host call ran");
    if let Err(cause) = finished {
        let expected = request.interruption().map(|interruption| match interruption {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled =>
                crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded =>
                crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption::DeadlineExceeded,
        });
        assert_eq!(
            Some(cause),
            expected.map(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Interrupted
            )
        );
        assert!(
            request.interruption().is_some(),
            "unexpected host refusal: {cause:?}"
        );
    }
    attempt
}
