//! An admitted computation and the projection custody it can seal.
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;

pub fn attempt_billed<Tested, Owned>(
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
    with_admitted_attempt_placed(
        world,
        installed,
        prior,
        request,
        placement,
        |_| {},
        |phase| admit(world, request, phase),
        ceiling,
        None,
    )
}

/// [`attempt_in`] with the request's execution opened under `placement`.
pub fn attempt_placed<Tested, Owned>(
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
pub fn attempt_placed_observing<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    before_computation: impl FnMut(&QueryRequestExecution<'_>),
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    with_admitted_attempt_placed(
        world,
        installed,
        prior,
        request,
        placement,
        before_computation,
        |phase| admit(world, request, phase),
        None,
        None,
    )
}

pub type Admission =
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation<
        Schema,
        TouchAccountOperation,
        crate::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput,
        Account,
    >;

/// Admission is fixture setup; the constrained host call pays the computation.
pub fn attempt_admitted_placed_observing<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    before_computation: impl FnMut(&QueryRequestExecution<'_>),
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    let admission = world
        .application
        .with_application_advancement(request, |phase| admit(world, request, &phase))
        .expect("fixture admission host call opens");
    with_admitted_attempt_placed(
        world,
        installed,
        prior,
        request,
        placement,
        before_computation,
        |_| admission,
        None,
        None,
    )
}

pub fn admit(
    world: &AuthorizationWorld,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
) -> Admission {
    let phase_custody = phase.test_host_request_for_test();
    let principal = authenticated_principal(&(phase_custody).test_host_request_for_test(), world, request);
    let account = resolved_account(&(phase_custody).test_host_request_for_test(), world, "open", request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &principal,
            &account,
            &operation,
            Default::default(),
            request,
        )
        .unwrap()
}

pub fn with_admitted_attempt_placed<Tested, Owned>(
    world: &AuthorizationWorld,
    installed: &WorthQueryInstalledPartitionedComputation<Schema, Feature, Tested, Owned>,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    mut before_computation: impl FnMut(&QueryRequestExecution<'_>),
    admit: impl FnOnce(
        &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Admission,
    ceiling: Option<u64>,
    kernel_receipts: Option<Arc<Mutex<Vec<(u64, u64)>>>>,
) -> Attempt
where
    Tested: ApplicationManagedComputation<Schema, Feature, Input = Input>,
    Owned: TestOwner<Tested>,
{
    let retained_prior = prior.as_ref().is_some_and(|prior| prior.retained.is_ok());
    let _placement =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::FixturePlacement::enter(placement);
    let mut observed = None;
    let finished = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request_for_test();
            let admission = admit(&phase);
            (*installed.owner_for_test()).gathered().lock().unwrap().clear();
            runs();
            tree_runs();
            ComputationPrior::hand_in_test_for_test(prior);
            SealedComputationRun::keep_in_test_for_test(None);
            let mut outcome = None;
            let mut bill = None;
            let completed = world.invariant.project_admitted_operation(
                &phase_custody.phase_for_application_for_test(&world.application),
                &admission,
                |reader, root| {
                    outcome = Some((|| -> Outcome {
                        let execution = phase
                            .execution_for_for_test(&(*world.application.product_runtime_for_test()))
                            .expect("the fixture phase belongs to its installed world");
                        before_computation(execution);
                        let prepared = installed.prepare_through_for_test(reader, execution, root)?;
                        let prepared = if let Some(receipts) = &kernel_receipts {
                            let receipts = Arc::clone(receipts);
                            prepared.observe_kernel_cost_for_test(move |work, span| {
                                receipts.lock().unwrap().push((work, span));
                            })
                        } else {
                            prepared
                        };
                        let compute = || {
                            prepared.compute(WorthQueryManagedComputationExecution::new(execution))
                        };
                        let (computed, report) = execution
                            .execution_request()
                            .unwrap()
                            .run(worth_execution::ExecutionWorkCeiling::new(ceiling), |_| compute())
                            .unwrap();
                        bill = Some(report);
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
                    if let Some(interruption) = request.interruption() {
                        assert_eq!(
                            denial.invariant_denial().and_then(|denial| denial.source_retention_interruption()),
                            Some(interruption),
                        );
                    } else {
                        // Retaining the fact after gather can exhaust this
                        // policy. Its allocation size is not part of the law.
                        assert!(matches!(
                            denial.invariant_denial().map(|denial| denial.kind()),
                            Some(worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                                worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                                    requested, admitted: 0, ancestor: 0,
                                }
                            )) if requested > 0
                        ), "unexpected fixture projection denial (retained prior: {retained_prior}): {denial:?}");
                    }
                    (
                        None,
                        denial.projection_work().expect("the reader performed work"),
                    )
                }
            };
            let outcome = outcome.expect("the admitted computation ran");
            ComputationPrior::hand_in_test_for_test(None);
            // A refused projection retains no sealed projection or computation run.
            let sealed = if let Some(projection) = projection {
                world
                    .application
                    .begin_projected_application_read_attempt(
                        &phase_custody.phase_for_application_for_test(&world.application),
                        admission,
                        projection,
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .map_err(|_| ())
                    .and_then(|attempt| attempt.complete_projected_dependencies(
                        &phase_custody.phase_for_application_for_test(&world.application),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    ).map_err(|_| ()))
                    .map(|_| SealedComputationRun::kept_in_test_for_test())
            } else {
                Err(())
            };
            let mut gathered = (*installed.owner_for_test()).gathered().lock().unwrap().clone();
            gathered.sort_unstable();
            observed = Some(Attempt {
                outcome,
                bill,
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
                worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded =>
                worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::DeadlineExceeded,
        });
        assert_eq!(
            Some(cause),
            expected.map(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted
            )
        );
        assert!(
            request.interruption().is_some(),
            "unexpected host refusal: {cause:?}"
        );
    }
    attempt
}

/// One billed attempt with its receiver-owned kernel completion receipt.
pub fn attempt_billed_with_receipts(
    world: &AuthorizationWorld,
    installed: &Installed,
    prior: Option<ComputationPrior>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    placement: RuntimeWorldExecutionPlacement<'_>,
    ceiling: u64,
    receipts: Arc<Mutex<Vec<(u64, u64)>>>,
) -> Attempt {
    with_admitted_attempt_placed(
        world,
        installed,
        prior,
        request,
        placement,
        |_| {},
        |phase| admit(world, request, phase),
        Some(ceiling),
        Some(receipts),
    )
}
