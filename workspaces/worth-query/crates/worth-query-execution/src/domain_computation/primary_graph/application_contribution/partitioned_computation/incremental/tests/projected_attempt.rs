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
    let completed = world.invariant.project_admitted_operation(
        &admission,
        |reader, root| {
            outcome = Some((|| -> Outcome {
                let execution = QueryRequestExecution::open(placement, request);
                let computed = installed
                    .prepare_through(reader, &execution, root)?
                    .compute(WorthQueryManagedComputationExecution::new(&execution))?;
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
        work,
        runs: runs(),
        tree_runs: tree_runs(),
        gathered,
        sealed,
    }
}
