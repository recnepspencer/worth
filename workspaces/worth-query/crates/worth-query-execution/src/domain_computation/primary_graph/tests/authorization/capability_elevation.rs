use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationOperationMarkerIdentity,
    StringApplicationValueBinding,
};

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{
    installed_elevated_capability_world, live_scope, AccountIdentity, CapabilityAction,
    CapabilityActionBinding, CapabilityDisclosure, CapabilityElevationIdentity,
    CapabilityElevationScenario, CapabilityElevationStatus, CapabilityPurpose,
    CapabilityPurposeBinding, CapabilityRequestContext, CapabilityTouchOperation,
    ElevatedCapabilityTouchInput, ElevatedCapabilityTouchOperation, ElevatedTouchAccountCapability,
    IdentityExecutionSchema, RequestCapabilityElevationOperation, RequestElevationInput,
    TouchAccountCapability,
};
use super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAuthorizationExplanationCause, WorthQueryApprovedElevation,
    WorthQueryOperationAuthorizationDenialKind,
};

#[path = "capability_elevation/active_use.rs"]
mod active_use;
#[path = "capability_elevation/approval_adversarial.rs"]
mod approval_adversarial;
#[path = "capability_elevation/approval_support_currentness.rs"]
mod approval_support_currentness;
#[path = "capability_elevation/approval_transition.rs"]
mod approval_transition;
#[path = "capability_elevation/approver_conflict.rs"]
mod approver_conflict;
#[path = "capability_elevation/close_provider_currentness.rs"]
mod close_provider_currentness;
#[path = "capability_elevation/close_transition.rs"]
mod close_transition;
#[path = "capability_elevation/delivery_cutoff.rs"]
mod delivery_cutoff;
#[path = "capability_elevation/mutation.rs"]
mod mutation;
#[path = "capability_elevation/request_commit_revalidation.rs"]
mod request_commit_revalidation;
#[path = "capability_elevation/request_support.rs"]
mod request_support;
#[path = "capability_elevation/request_transition.rs"]
mod request_transition;
#[path = "capability_elevation/review_support_mutation.rs"]
mod review_support_mutation;
#[path = "capability_elevation/review_transition.rs"]
mod review_transition;
#[path = "capability_elevation/terminal_lifecycle_support.rs"]
mod terminal_lifecycle_support;
#[path = "capability_elevation/terminal_state.rs"]
mod terminal_state;
#[path = "capability_elevation/validity.rs"]
mod validity;

struct AliasedRequestCapabilityElevationOperation;

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
fn exact_active_elevation_admits_and_revalidates_with_ordinary_capability_authority() {
    let (world, request, approved) = approval_transition::exact_approved_world();
    assert_eq!(
        world
            .application
            .capability_plan_compilation_evidence()
            .elevation_lifecycle_operation_count(),
        4
    );
    let (_, _, request_role) = world
        .application
        .authorization
        .elevation_lifecycle_operation::<IdentityExecutionSchema, RequestCapabilityElevationOperation>(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .expect("the request marker must match installed lifecycle identity")
        .expect("the request operation must have one lifecycle owner");
    assert_eq!(format!("{request_role:?}"), "Request");
    assert!(world
        .application
        .authorization
        .elevation_lifecycle_operation::<
            IdentityExecutionSchema,
            AliasedRequestCapabilityElevationOperation,
        >(
            "RequestCapabilityElevationOperation",
            <RequestElevationInput as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str(),
        )
        .is_err());
    world.authorization_time.script([time(100), time(100)]);
    let principal = authenticated_principal(&world, &request);
    let access = admit(&world, &approved, &principal, &request, Some("elevation-2")).unwrap();

    assert_eq!(access.authorization_decision_fact_count(), 2);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();
    world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .expect("the exact active elevation must survive fresh operation re-admission");
}

#[test]
fn selector_only_admission_cannot_bypass_the_approval_transition() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admit_raw(&world, &principal, &request, Some("elevation-1")) else {
        panic!("a raw selector must not substitute for approved lifecycle authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
    );
}

#[test]
fn governed_capability_without_elevation_preserves_required_explanation() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admit_raw(&world, &principal, &request, None) else {
        panic!("a governed capability cannot omit elevation")
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationRequired
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ElevationRequired)
    );
}

#[test]
fn resource_selector_cannot_substitute_for_the_declared_elevation_identity() {
    let (world, request, approved) = approval_transition::exact_approved_world();
    let principal = authenticated_principal(&world, &request);
    let capability = installed_capability(&world);
    let mut input = elevated_input(Some("elevation-2"));
    input.substitute_resource_selector = true;

    let Err(denial) = world.selected_product().admit_approved_elevation_access(
        &approved,
        &principal,
        &capability,
        input,
        &request,
    ) else {
        panic!("a selector for another installed entity kind must not open elevation authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationProjectionRejected
    );
}

#[test]
fn non_governed_capability_rejects_an_elevation_selector() {
    let world = installed_elevated_capability_world(CapabilityElevationScenario::Active);
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let projection = ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            AccountIdentity::reference(),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "account-1".to_owned(),
            )
            .expect("fixture account identity must encode"),
        ),
        ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(CapabilityAction::Touch)
            .expect("fixture capability action must encode"),
        ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
            CapabilityPurpose::AccountMaintenance,
        )
        .expect("fixture capability purpose must encode"),
        ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
    )
    .elevation(ApplicationCapabilityEntitySelector::new(
        CapabilityElevationIdentity::reference(),
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
            "elevation-1".to_owned(),
        )
        .expect("fixture elevation identity must encode"),
    ));

    let denial = crate::domain_computation::authorization::validate_elevation_projection(
        capability.contract(),
        &projection,
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationNotApplicable
    );
}

#[test]
fn revoked_approved_elevation_cannot_open_active_authority() {
    let (world, request, approved) = approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = authenticated_principal(&world, &request);
    mutation::set_status(&world, "elevation-2", CapabilityElevationStatus::Revoked);

    let Err(denial) = admit(&world, &approved, &principal, &request, Some("elevation-2")) else {
        panic!("revoked approved authority must not open active use");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationInactive
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::ElevationDenied)
    );
}

#[test]
fn elevation_status_drift_after_admission_is_stale_at_operation_progression() {
    let (world, request, approved) = approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = authenticated_principal(&world, &request);
    let access = admit(&world, &approved, &principal, &request, Some("elevation-2")).unwrap();
    mutation::set_status(&world, "elevation-2", CapabilityElevationStatus::Revoked);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) =
        world
            .application
            .authorize_capability_operation(access, &operation, Default::default())
    else {
        panic!("revoked elevation evidence must not progress to operation authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn approver_drift_after_admission_is_stale_before_operation_authority() {
    let (world, request, approved) = approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = authenticated_principal(&world, &request);
    let access = admit(&world, &approved, &principal, &request, Some("elevation-2")).unwrap();
    mutation::add_self_approver(&world, "elevation-2", principal.principal_entity_id());
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) =
        world
            .application
            .authorize_capability_operation(access, &operation, Default::default())
    else {
        panic!("changed exact approver relationships must stale admitted elevation evidence");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

fn admit(
    world: &super::super::fixture::AuthorizationWorld,
    approved: &WorthQueryApprovedElevation,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        super::super::fixture::IdentityExecutionSchema,
        super::super::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: Option<&str>,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        super::super::fixture::IdentityExecutionSchema,
        ElevatedTouchAccountCapability,
        ElevatedCapabilityTouchOperation,
        ElevatedCapabilityTouchInput,
    >,
    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial,
> {
    let capability = installed_capability(world);
    world.selected_product().admit_approved_elevation_access(
        approved,
        principal,
        &capability,
        elevated_input(elevation),
        request,
    )
}

fn admit_raw(
    world: &super::super::fixture::AuthorizationWorld,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        super::super::fixture::IdentityExecutionSchema,
        super::super::fixture::Principal,
        u64,
    >,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    elevation: Option<&str>,
) -> Result<
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
        super::super::fixture::IdentityExecutionSchema,
        ElevatedTouchAccountCapability,
        ElevatedCapabilityTouchOperation,
        ElevatedCapabilityTouchInput,
    >,
    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial,
> {
    let capability = installed_capability(world);
    world.selected_product().admit_capability_access(
        principal,
        &capability,
        elevated_input(elevation),
        request,
    )
}

fn installed_capability(
    world: &super::super::fixture::AuthorizationWorld,
) -> worth_query_installation::facade::WorthQueryInstalledApplicationCapability<
    super::super::fixture::IdentityExecutionSchema,
    ElevatedTouchAccountCapability,
    ElevatedCapabilityTouchOperation,
    ElevatedCapabilityTouchInput,
> {
    world
        .application
        .installed_schema()
        .capability(
            ElevatedTouchAccountCapability::reference(),
            ElevatedCapabilityTouchOperation::reference(),
        )
        .unwrap()
}

fn elevated_input(elevation: Option<&str>) -> ElevatedCapabilityTouchInput {
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
