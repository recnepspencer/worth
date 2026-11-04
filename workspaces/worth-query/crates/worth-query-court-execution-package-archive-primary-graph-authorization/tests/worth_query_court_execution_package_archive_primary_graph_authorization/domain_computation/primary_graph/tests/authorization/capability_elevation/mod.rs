pub use worth_query_declaration::facade::application_capability::ApplicationCapabilityEntitySelector;
pub use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestContext;
pub use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
pub use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
pub use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
pub use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_elevated_capability_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityActionBinding;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityElevationIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityPurposeBinding;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityRequestContext;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedCapabilityTouchOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ElevatedTouchAccountCapability;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RequestCapabilityElevationOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
pub use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAuthorizationExplanationCause;
pub use worth_query_execution::facade::domain_computation::WorthQueryApprovedElevation;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

pub struct AliasedRequestCapabilityElevationOperation;

impl ApplicationOperationMarkerIdentity<IdentityExecutionSchema>
    for AliasedRequestCapabilityElevationOperation
{
    type InputBinding =
        <RequestCapabilityElevationOperation as ApplicationOperationMarkerIdentity<
            IdentityExecutionSchema,
        >>::InputBinding;
    const IDENTIFIER: &'static str = "AliasedRequestCapabilityElevationOperation";
}

#[test]
fn governed_capability_without_elevation_preserves_required_explanation() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| admit_raw(&phase, &world, &principal, &request, None))
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a governed capability cannot omit elevation")
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationRequired
        )
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ElevationRequired)
    );
}

#[test]
fn resource_selector_cannot_substitute_for_the_declared_elevation_identity() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let capability = installed_capability(&world);
    let mut input = elevated_input(Some("elevation-2"));
    input.substitute_resource_selector = true;

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .selected_product(&host_custody)
                .admit_approved_elevation_access(
                    &host_custody.phase_for_application(&world.application),
                    &approved,
                    &principal,
                    &capability,
                    input,
                    &request,
                )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a selector for another installed entity kind must not open elevation authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationProjectionRejected
        )
    );
}

#[test]
fn revoked_approved_elevation_cannot_open_active_authority() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
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

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("revoked approved authority must not open active use");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationInactive
        )
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ElevationDenied)
    );
}

#[test]
fn approver_drift_after_admission_is_stale_before_operation_authority() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            authenticated_principal(&host_custody, &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            admit(
                &host_custody,
                &world,
                &approved,
                &principal,
                &request,
                Some("elevation-2"),
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::mutation::add_self_approver(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world,
                "elevation-2",
                principal.principal_entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world.application.authorize_capability_operation(
                &host_custody.phase_for_application(&world.application),
                access,
                &operation,
                Default::default(),
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("changed exact approver relationships must stale admitted elevation evidence");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization
        )
    );
}

pub fn admit(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    approved: &WorthQueryApprovedElevation,
    principal: &worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: Option<&str>,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        ElevatedTouchAccountCapability,
        ElevatedCapabilityTouchOperation,
        ElevatedCapabilityTouchInput,
    >,
    worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial,
>{
    let capability = installed_capability(world);
    world
        .selected_product(phase_custody)
        .admit_approved_elevation_access(
            &phase_custody.phase_for_application_for_test(&world.application),
            approved,
            principal,
            &capability,
            elevated_input(elevation),
            request,
        )
}

pub fn admit_raw(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    principal: &worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: Option<&str>,
) -> Result<
    worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        ElevatedTouchAccountCapability,
        ElevatedCapabilityTouchOperation,
        ElevatedCapabilityTouchInput,
    >,
    worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial,
>{
    let capability = installed_capability(world);
    world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .admit_capability_access(
            phase_custody,
            principal,
            &capability,
            elevated_input(elevation),
            request,
        )
}

pub fn installed_capability(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationCapability<
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
    ElevatedTouchAccountCapability,
    ElevatedCapabilityTouchOperation,
    ElevatedCapabilityTouchInput,
>{
    world
        .application
        .installed_schema()
        .capability(
            ElevatedTouchAccountCapability::reference(),
            ElevatedCapabilityTouchOperation::reference(),
        )
        .unwrap()
}

pub fn elevated_input(elevation: Option<&str>) -> ElevatedCapabilityTouchInput {
    ElevatedCapabilityTouchInput {
        account: "account-1".to_owned(),
        elevation: elevation.map(str::to_owned),
        substitute_resource_selector: false,
        action: CapabilityAction::Touch,
        purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: CapabilityDisclosure::AccountActivity,
        amount: 50,
    }
}

#[path = "active_use/mod.rs"]
pub mod active_use;
#[path = "approval_adversarial/mod.rs"]
pub mod approval_adversarial;
#[path = "approval_support_currentness/mod.rs"]
pub mod approval_support_currentness;
#[path = "approval_transition/mod.rs"]
pub mod approval_transition;
#[path = "approver_conflict/mod.rs"]
pub mod approver_conflict;
#[path = "close_provider_currentness/mod.rs"]
pub mod close_provider_currentness;
#[path = "close_transition/mod.rs"]
pub mod close_transition;
#[path = "delivery_cutoff/mod.rs"]
pub mod delivery_cutoff;
#[path = "elevation_status_drift_after_admission_is_stale_at_operation_pro/mod.rs"]
pub mod elevation_status_drift_after_admission_is_stale_at_operation_pro;
#[path = "exact_active_elevation_admits_and_revalidates_with_ordinary_capa/mod.rs"]
pub mod exact_active_elevation_admits_and_revalidates_with_ordinary_capa;
#[path = "mutation/mod.rs"]
pub mod mutation;
#[path = "non_governed_capability_rejects_an_elevation_selector/mod.rs"]
pub mod non_governed_capability_rejects_an_elevation_selector;
#[path = "request_commit_revalidation/mod.rs"]
pub mod request_commit_revalidation;
#[path = "request_support/mod.rs"]
pub mod request_support;
#[path = "request_transition/mod.rs"]
pub mod request_transition;
#[path = "review_support_mutation/mod.rs"]
pub mod review_support_mutation;
#[path = "review_transition/mod.rs"]
pub mod review_transition;
#[path = "selector_only_admission/mod.rs"]
pub mod selector_only_admission;
#[path = "terminal_lifecycle_support/mod.rs"]
pub mod terminal_lifecycle_support;
#[path = "terminal_state/mod.rs"]
pub mod terminal_state;
#[path = "validity/mod.rs"]
pub mod validity;
