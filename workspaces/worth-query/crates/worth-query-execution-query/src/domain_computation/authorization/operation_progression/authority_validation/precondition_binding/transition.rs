use super::PreconditionBoundCapabilityOperation;
use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;

pub use crate::domain_computation::authorization::operation_admission as admitted_operation;
pub use admitted_operation::{
    authorize_applied_control_mutation_on_selected, authorize_current_control_mutation_on_selected,
    authorize_historical_control_mutation_on_selected, authorize_public_mutation_on_selected,
    SelectedConventionalAdmissionStop,
};

pub use admitted_operation::transition_conventional_operation;
pub use admitted_operation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use admitted_operation::WorthQueryAdmittedApplicationOperation;
pub use admitted_operation::WorthQueryOperationAdmissionIdentity;
pub use admitted_operation::{
    admit_capability_access, admit_encoded_capability_access,
};
pub use admitted_operation::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

pub fn transition_capability_operation<Schema, Capability, Operation, Input>(
execution: &worth_execution::ExecutionRequest<'_, '_>,
    bound: PreconditionBoundCapabilityOperation<'_, Schema, Capability, Operation, Input>,
) -> Result<
    WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        <Input as worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Input: worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
        Schema,
        Capability,
    >,
{
    let PreconditionBoundCapabilityOperation {
        validated,
        preconditions,
    } = bound;
    let super::super::ValidatedCapabilityOperation {
        runtime,
        access,
        operation,
    } = validated;
    access.transition_operation(execution, runtime, operation, preconditions)
}

pub use admitted_operation::{
    BrandTag, ReadCompletionAdmission, WorthQueryAdmittedOperation,
};
