pub use std::time::Duration;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_elevated_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ApproveCapabilityElevationOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ApproveElevationCapability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveElevationInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationApprover;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationGrant;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationNotAfter;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationNotBefore;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationReason;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationRequester;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationResource;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationReview;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewKindField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewResource;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewStatusField;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewer;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use worth_query_execution::facade::domain_computation::WorthQueryApprovedElevation;
pub use worth_query_execution::facade::domain_computation::WorthQueryElevationApprovalOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryElevationApprovalProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::domain_computation::WorthQueryRequestedElevation;

pub type World = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub type Authenticated =
    worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >;
pub type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    ApproveElevationCapability,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
>;

#[test]
fn exact_request_approval_reobserves_lifecycle_and_commits_two_derived_effects() {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            materialize_exact_approval(&host_custody, &world, &request, requested)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryElevationApprovalOutcome::Approved(approved) = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    else {
        panic!("the exact approval transition must commit");
    };
    assert_approved(&approved, &approver);
}

pub fn requested_world(
    scenario: CapabilityElevationScenario,
) -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryRequestedElevation,
) {
    requested_world_with_input(scenario, crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_transition::honest_input())
}

pub fn requested_world_with_input(
    scenario: CapabilityElevationScenario,
    input: worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationInput,
) -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryRequestedElevation,
) {
    let world = installed_elevated_capability_world(scenario);
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let requested = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_support::commit_request(&world, &request, input);
    crate::domain_computation::primary_graph::tests::authorization::capability_elevation::request_support::resolve_exact_request_identities(&world, &request);
    (world, request, requested)
}

pub fn authenticated(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    subject: &str,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Authenticated {
    let borrowed_execution = phase_custody.borrowed_request_for_test();
    let execution = &borrowed_execution;

    let external = world.authenticate(subject, Duration::from_secs(60), request);
    world
        .application
        .select_product_branch_for_test(
            execution,
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &phase_custody.phase_for_application_for_test(&world.application),
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub fn approval_access(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<
    Access,
    worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial,
> {
    approval_access_for_elevation(
        &(phase_custody).test_host_request_for_test(),
        world,
        principal,
        request,
        "elevation-2",
    )
}

pub fn approval_access_for_elevation(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &Authenticated,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: &str,
) -> Result<
    Access,
    worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial,
> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            ApproveElevationCapability::reference(),
            ApproveCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application_for_test(&world.application),
            principal,
            &capability,
            ApproveElevationInput {
                account: "account-1".to_owned(),
                elevation: elevation.to_owned(),
            },
            request,
        )
}

pub fn approval_operation(
    world: &World,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
> {
    world
        .application
        .installed_schema()
        .installed_operation(ApproveCapabilityElevationOperation::reference())
        .unwrap()
}

pub fn seal_approval_facts(
    reader: &mut worth_query_execution::facade::domain_computation::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        ApproveCapabilityElevationOperation,
    >,
) {
    let elevation = reader
        .resolve_entity(
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
        )
        .unwrap();
    let review = reader
        .resolve_entity(CapabilityReviewIdentity::reference(), "review-2".to_owned())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationReason::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationStatusField::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationNotBefore::reference())
        .unwrap();
    reader
        .require_decision_field(&elevation, CapabilityElevationNotAfter::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewKindField::reference())
        .unwrap();
    reader
        .require_decision_field(&review, CapabilityReviewStatusField::reference())
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationRequester::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationApprover::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationGrant::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationResource::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationReview::reference(), &elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityReviewResource::reference(), &review)
        .unwrap();
    reader
        .decision_relations_to(CapabilityReviewer::reference(), &review)
        .unwrap();
}

pub fn assert_approved(approved: &WorthQueryApprovedElevation, approver: &Authenticated) {
    assert_eq!(approved.approval_changed_record_count(), 3);
    assert_eq!(approved.approval_emitted_effect_count(), 0);
    assert_eq!(approved.approver(), approver.principal_entity_id_for_test());
    assert_ne!(approved.requester(), approved.approver());
    assert!(approved.commits_share_branch());
}

pub fn exact_approved_world() -> (
    World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    WorthQueryApprovedElevation,
) {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approved = approve_exact_request(&world, &request, requested);
    (world, request, approved)
}

pub fn approve_exact_request(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> WorthQueryApprovedElevation {
    let program = world
        .application
        .with_application_advancement(request, |phase| {
            let phase_custody = phase.test_host_request_for_test();
            materialize_exact_approval(
                &(phase_custody).test_host_request_for_test(),
                world,
                request,
                requested,
            )
        })
        .expect("the approval materialization owns its host call");
    match world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    {
        WorthQueryElevationApprovalOutcome::Approved(approved) => approved,
        unexpected => panic!("the canonical approval prerequisite must commit: {unexpected:?}"),
    }
}

pub fn materialize_exact_approval(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    requested: WorthQueryRequestedElevation,
) -> WorthQueryElevationApprovalProgram<
    IdentityExecutionSchema,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
    Account,
> {
    let approver = authenticated(phase_custody, world, "bob", request);
    let access = approval_access(phase_custody, world, &approver, request).unwrap();
    let operation = approval_operation(world);
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
            |reader, _| seal_approval_facts(reader),
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
        .materialize_elevation_approval_program()
        .unwrap()
}

pub fn string(value: &str) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(value),
    )
}

#[path = "approval_after_the_exact_request_window_returns_the_request_rece/mod.rs"]
pub mod approval_after_the_exact_request_window_returns_the_request_rece;
#[path = "approval_command_cannot_select_a_different_elevation_subject/mod.rs"]
pub mod approval_command_cannot_select_a_different_elevation_subject;
#[path = "approval_refusal/mod.rs"]
pub mod approval_refusal;
