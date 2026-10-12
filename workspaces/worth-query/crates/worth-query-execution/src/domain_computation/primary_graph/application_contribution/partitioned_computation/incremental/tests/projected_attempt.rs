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
    attempt_billed(world, installed, prior, request, placement, None)
}

pub(super) fn attempt_billed<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    ceiling: Option<u64>,
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    let principal = authenticated_principal(world, request);
    let account = resolved_account(world, "open", request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
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
    let mut bill = None;
    let completed = world.invariant.project_admitted_operation(
        &admission,
        |reader, root| {
            outcome = Some((|| -> Outcome {
                let execution = QueryRequestExecution::open(placement, request);
                let prepared = installed.prepare_through(reader, &execution, root)?;
                let compute =
                    || prepared.compute(WorthQueryManagedComputationExecution::new(&execution));
                let computed = if let Some(ceiling) = ceiling {
                    let (computed, report) = execution
                        .execution_request()
                        .unwrap()
                        .run(worth_execution::ExecutionWorkCeiling::new(ceiling), |_| {
                            compute()
                        })
                        .unwrap();
                    bill = Some(report);
                    computed
                } else {
                    compute()
                };
                let computed = computed?;
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
                admission,
                projection.expect("an uninterrupted projection sealed"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .complete_projected_dependencies(
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map(|_| SealedComputationRun::kept_in_test())
            .map_err(|_| ())
    };
    let mut gathered = installed.owner.gathered().lock().unwrap().clone();
    gathered.sort_unstable();
    Attempt {
        outcome,
        bill: bill.unwrap_or_else(|| {
            ExecutionReport::new(
                worth_foundational::ExecutionPosture::Serial,
                0,
                0,
                Default::default(),
            )
        }),
        work,
        runs: runs(),
        tree_runs: tree_runs(),
        gathered,
        sealed,
    }
}
