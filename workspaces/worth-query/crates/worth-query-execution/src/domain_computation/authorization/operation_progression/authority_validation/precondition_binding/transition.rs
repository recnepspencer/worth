use super::PreconditionBoundCapabilityOperation;

#[path = "../../../operation_admission.rs"]
mod admitted_operation;

pub(in crate::domain_computation) use admitted_operation::admit_capability_access;
pub(in crate::domain_computation::authorization::operation_progression) use admitted_operation::transition_conventional_operation;
pub use admitted_operation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use admitted_operation::WorthQueryAdmittedApplicationOperation;
pub(in crate::domain_computation) use admitted_operation::WorthQueryOperationAdmissionIdentity;
pub(in crate::domain_computation::authorization) use admitted_operation::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

pub(in crate::domain_computation::authorization::operation_progression) fn transition_capability_operation<Schema, Capability, Operation, Input>(
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
    access.transition_operation(runtime, operation, preconditions)
}
