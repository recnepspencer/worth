use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ApproveCapabilityElevationOperation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveElevationInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryCompleteApplicationReadSet;
use worth_query_execution::facade::domain_computation::WorthQueryElevationApprovalOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryProjectedApplicationMutation;
use worth_query_execution::facade::domain_computation::WorthQueryRequestedElevation;

#[test]
fn any_preexisting_approver_relation_blocks_fresh_approval_materialization() {
    let (world, request, requested) =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let requester = requested.requester();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::mutation::add_self_approver(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                requester,
            )
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            approval_reads(&host_custody, &world, &request, requested)
                .materialize_elevation_approval_program()
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("approval must prove the complete preexisting approver set is empty");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ElevationApprovalProgramMismatch
    );
}

pub type Reads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

#[test]
fn ordinary_operation_and_commit_paths_cannot_publish_approval_authority() {
    let (world, request, requested) =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_access(&host_custody, &world, &approver, &request)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_operation(&world);
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .authorize_capability_operation(
                    &host_custody.phase_for_application_for_test(&world.application),
                    access,
                    &operation,
                    Default::default(),
                )
                .err()
                .expect("ordinary operation progression must reject approval lifecycle work")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationTransitionRequired
        )
    );

    let ordinary = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            approval_reads(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                &world,
                &request,
                requested,
            )
            .begin_effect_program()
            .finish()
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            ordinary,
            idempotency(173, 173),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("ordinary compare-and-commit must reject approval lifecycle authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn lifecycle_drift_before_approval_commit_is_stale_and_returns_request_authority() {
    let (world, request, requested) =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            approval_reads(&host_custody, &world, &request, requested)
                .materialize_elevation_approval_program()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::mutation::set_status(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                CapabilityElevationStatus::Revoked,
            )
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(174, 174));
    let WorthQueryElevationApprovalOutcome::Denied(denial, requested) = outcome else {
        panic!("the prior-product approval must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(requested.elevation_identity(), &string("elevation-2"));
}

pub fn approval_reads(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> Reads {
    let approver = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(phase_custody, world, "bob", request);
    let access =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_access(phase_custody, world, &approver, request)
            .unwrap();
    let operation = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::approval_operation(world);
    let admission = world
        .application
        .authorize_elevation_approval(
            &phase_custody.phase_for_application_for_test(&world.application),
            requested,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &admission,
            |reader, _| crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::seal_approval_facts(reader),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

pub fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}
