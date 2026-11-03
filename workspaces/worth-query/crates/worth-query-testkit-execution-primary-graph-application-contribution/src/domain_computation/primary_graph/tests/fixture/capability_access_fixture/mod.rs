use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityTouchInput;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;

pub fn admit_touch_account_capability(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &WorthQueryRequestScope,
) -> Result<
    WorthQueryAdmittedApplicationCapabilityAccess<
        IdentityExecutionSchema,
        TouchAccountCapability,
        CapabilityTouchOperation,
        CapabilityTouchInput,
    >,
    WorthQueryOperationAuthorizationDenial,
> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(phase_custody)
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            CapabilityTouchInput {
                account: "account-1".to_owned(),
                action: CapabilityAction::Touch,
                purpose: CapabilityPurpose::AccountMaintenance,
                disclosure: CapabilityDisclosure::AccountActivity,
                related_account: "account-2".to_owned(),
                amount: 50,
                caller_time: 100,
                request_record: "selected-request".to_owned(),
                prior_record: "selected-prior".to_owned(),
            },
            request,
        )
}
