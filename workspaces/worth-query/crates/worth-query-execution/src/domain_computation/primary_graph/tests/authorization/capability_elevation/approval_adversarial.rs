use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{
    Account, ApproveCapabilityElevationOperation, ApproveElevationInput,
    CapabilityElevationScenario, CapabilityElevationStatus, IdentityExecutionSchema,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryCompleteApplicationReadSet, WorthQueryElevationApprovalOutcome,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryProjectedApplicationMutation,
    WorthQueryRequestedElevation,
};

#[test]
fn any_preexisting_approver_relation_blocks_fresh_approval_materialization() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let requester = requested.requester();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::mutation::add_self_approver(
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
            let host_custody = phase.test_host_request();
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

type Reads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

#[test]
fn ordinary_operation_and_commit_paths_cannot_publish_approval_authority() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::approval_transition::approval_access(&host_custody, &world, &approver, &request)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = super::approval_transition::approval_operation(&world);
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .authorize_capability_operation(
                    &host_custody.phase_for_application(&world.application),
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
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );

    let ordinary = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            approval_reads(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
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
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            approval_reads(&host_custody, &world, &request, requested)
                .materialize_elevation_approval_program()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            super::mutation::set_status(
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

fn approval_reads(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &super::approval_transition::World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> Reads {
    let approver = super::approval_transition::authenticated(&phase_custody, world, "bob", request);
    let access =
        super::approval_transition::approval_access(&phase_custody, world, &approver, request)
            .unwrap();
    let operation = super::approval_transition::approval_operation(world);
    let admission = world
        .application
        .authorize_elevation_approval(
            &phase_custody.phase_for_application(&world.application),
            requested,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application(&world.application),
            &admission,
            |reader, _| super::approval_transition::seal_approval_facts(reader),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}
