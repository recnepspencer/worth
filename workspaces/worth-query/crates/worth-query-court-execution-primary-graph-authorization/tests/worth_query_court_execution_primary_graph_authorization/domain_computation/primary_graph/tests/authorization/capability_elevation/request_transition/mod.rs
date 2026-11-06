pub use std::time::Duration;
pub use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_elevated_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityReviewIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RequestCapabilityElevationOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RequestElevationCapability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationInput;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryCompleteApplicationReadSet;
pub use worth_query_execution::facade::domain_computation::WorthQueryElevationRequestOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::domain_computation::WorthQueryProjectedApplicationMutation;

pub type World = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub type Principal = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    RequestElevationCapability,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
>;
pub type Reads = WorthQueryCompleteApplicationReadSet<
    IdentityExecutionSchema,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
    Account,
    WorthQueryProjectedApplicationMutation,
>;

#[test]
fn ordinary_compare_and_commit_cannot_publish_a_lifecycle_program() {
    let world = request_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let ordinary = world
        .application
        .with_host_advancement(|phase| {
            request_reads(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &request,
                honest_input(),
            )
            .begin_effect_program()
            .finish()
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            ordinary,
            idempotency(72, 72),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("ordinary compare-and-commit must reject lifecycle authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn request_upper_bound_cannot_widen_scope_or_swap_purpose() {
    for (input, expected) in [
        (
            RequestElevationInput {
                target_account: "account-2".to_owned(),
                ..honest_input()
            },
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityGrantMissing,
            ),
        ),
        (
            RequestElevationInput {
                target_purpose: CapabilityPurpose::Audit,
                ..honest_input()
            },
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationRequestRejected,
            ),
        ),
    ] {
        let world = request_world();
        let request = live_scope();
        world
            .application
            .with_host_advancement(|phase| {
                let principal = authenticated_principal(
                    &(phase).test_host_request_for_test(),
                    &world,
                    &request,
                );
                let access = request_access(
                    &(phase).test_host_request_for_test(),
                    &world,
                    &principal,
                    &request,
                    input,
                )
                .unwrap();
                let operation = request_operation(&world);

                let denial = world
                    .application
                    .authorize_elevation_request(&phase, access, &operation, Default::default())
                    .err()
                    .expect("a widened request target must not mint lifecycle authority");
                assert_eq!(denial.kind(), expected);
            })
            .expect("the request fixture owns its installed host call");
    }
}

#[test]
fn proposed_grant_must_independently_authorize_the_governed_upper_bound() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::WrongGrant);
    world.authorization_time.script([time(100), time(100)]);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();

            let principal = authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &request,
            );
            let access = request_access(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &principal,
                &request,
                RequestElevationInput {
                    grant: "capability-2".to_owned(),
                    ..honest_input()
                },
            )
            .unwrap();

            let denial = world
                .application
                .authorize_elevation_request(
                    &host_custody.phase_for_application_for_test(&world.application),
                    access,
                    &request_operation(&world),
                    Default::default(),
                )
                .err()
                .expect("a different grant cannot become the elevation upper bound");
            assert_eq!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::CapabilityGrantMissing
                )
            );
            assert!(denial.identity().is_some());
            assert_eq!(
                denial.causes(),
                [WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::CapabilityGrantMissing
                )]
            );
        })
        .expect("the fixture block owns its installed host call")
}

pub fn request_reads(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> Reads {
    let operation = request_operation(world);
    let access = request_access(phase_custody, world, principal, request, input).unwrap();
    let admission = world
        .application
        .authorize_elevation_request(
            &phase_custody.phase_for_application_for_test(&world.application),
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
            |reader, account| {
                reader
                    .require_decision_field(account, AccountLabel::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

pub fn request_access(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        IdentityExecutionSchema,
        Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: RequestElevationInput,
) -> Result<Access, WorthQueryOperationAuthorizationDenial> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            RequestElevationCapability::reference(),
            RequestCapabilityElevationOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_capability_access(
            &phase_custody.phase_for_application_for_test(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub fn request_operation(
    world: &World,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
    IdentityExecutionSchema,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
> {
    world
        .application
        .installed_schema()
        .installed_operation(RequestCapabilityElevationOperation::reference())
        .unwrap()
}

pub fn request_world() -> World {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.hold(time(100));
    world
}

pub fn honest_input() -> RequestElevationInput {
    RequestElevationInput {
        account: "account-1".to_owned(),
        target_account: "account-1".to_owned(),
        grant: "capability-1".to_owned(),
        elevation_key: "elevation-request-2".to_owned(),
        elevation_identity: "elevation-2".to_owned(),
        review_key: "review-request-2".to_owned(),
        review_identity: "review-2".to_owned(),
        reason: "protect-customer".to_owned(),
        duration: Duration::from_secs(5),
        action: CapabilityAction::Touch,
        target_purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: CapabilityDisclosure::AccountActivity,
        amount: 50,
    }
}

pub fn resolve_created_identities(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) {
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
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            CapabilityElevationIdentity::reference(),
            "elevation-2".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("committed request must create the exact elevation identity");
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
        .resolve_entity(
            &phase_custody.phase_for_application_for_test(&world.application),
            CapabilityReviewIdentity::reference(),
            "review-2".to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("committed request must create the exact review identity");
}

pub fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}

#[path = "equivalent_request_retry_recovers_the_same_authoritative_commit/mod.rs"]
pub mod equivalent_request_retry_recovers_the_same_authoritative_commit;
#[path = "exact_request_commits_query_derived_state_and_returns_one_reques/mod.rs"]
pub mod exact_request_commits_query_derived_state_and_returns_one_reques;
#[path = "installed_seal/mod.rs"]
pub mod installed_seal;
#[path = "ordinary_progression_refusal/mod.rs"]
pub mod ordinary_progression_refusal;
#[path = "zero_or_overlong_duration_and_invalid_storage_key_fail_before_co/mod.rs"]
pub mod zero_or_overlong_duration_and_invalid_storage_key_fail_before_co;
