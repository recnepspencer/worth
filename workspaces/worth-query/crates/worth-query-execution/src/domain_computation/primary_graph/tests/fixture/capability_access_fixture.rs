use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use super::world_installation::AuthorizationWorld;
use super::{
    CapabilityAction, CapabilityDisclosure, CapabilityGovernedInputIdentity, CapabilityPurpose,
    CapabilityTouchInput, CapabilityTouchOperation, IdentityExecutionSchema, Principal,
    TouchAccountCapability,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAuthenticatedPrincipal,
    WorthQueryOperationAuthorizationDenial,
};

pub(in crate::domain_computation::primary_graph) fn admit_touch_account_capability(
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
    world.selected_product().admit_capability_access(
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
            governed_input_identity: CapabilityGovernedInputIdentity::None,
        },
        request,
    )
}
