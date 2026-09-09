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
    super::mutation::add_self_approver(&world, "elevation-2", requester);

    let Err(denial) =
        approval_reads(&world, &request, requested).materialize_elevation_approval_program()
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
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::approval_transition::approval_access(&world, &approver, &request).unwrap();
    let operation = super::approval_transition::approval_operation(&world);
    let denial = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .err()
        .expect("ordinary operation progression must reject approval lifecycle work");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );

    let ordinary = approval_reads(&world, &request, requested)
        .begin_effect_program()
        .finish()
        .unwrap();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(ordinary, idempotency(173, 173))
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
    let program = approval_reads(&world, &request, requested)
        .materialize_elevation_approval_program()
        .unwrap();
    super::mutation::set_status(&world, "elevation-2", CapabilityElevationStatus::Revoked);

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
    world: &super::approval_transition::World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> Reads {
    let approver = super::approval_transition::authenticated(world, "bob", request);
    let access = super::approval_transition::approval_access(world, &approver, request).unwrap();
    let operation = super::approval_transition::approval_operation(world);
    let admission = world
        .application
        .authorize_elevation_approval(requested, access, &operation, Default::default())
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            super::approval_transition::seal_approval_facts(reader)
        })
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
}

fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}
