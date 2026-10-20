//! A capability-governed mutation binding with a handler that accepts the
//! input it is given. It shows that the handler runs on exactly the input its
//! capability admission governed, and that a request whose input differs is
//! refused before any handler runs.

use super::capability::CapabilityTouchOperationInputBinding;
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, NoApplicationMutationOutputs,
};
use worth_query_declaration::facade::application_schema::{
    NoApplicationUnit, ReadWrite, U64ApplicationValueBinding,
};

worth_query_declaration::worth_query_mutation_binding!(
    pub CapabilityTouchMutationBinding for CapabilityTouchInput,
    schema IdentityExecutionSchema,
    identity "worth.query.test.capability-touch-mutation.v1",
    input CapabilityTouchOperationInputBinding,
    operation CapabilityTouchOperation,
    result CapabilityTouchOperationInputBinding,
    idempotency String, identity "worth.query.test.capability-touch-key.v1",
    decision CapabilityTouchInput,
    denial CapabilityTouchOperationInputBinding,
    handler identity "worth.query.test.capability-touch-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding,
        mapping ExternalMapping,
        principal_entity Principal,
        principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(),
    value touch_scope,
);

fn touch_scope(input: &CapabilityTouchInput) -> String {
    input.account.clone()
}

impl ApplicationCapabilityMutationBinding<IdentityExecutionSchema>
    for CapabilityTouchMutationBinding
{
    type Capability = TouchAccountCapability;
}

/// Accepts the admitted input unchanged and stages no write.
pub(super) struct CapabilityTouchHandler;

impl OperationHandler<IdentityExecutionSchema, CapabilityTouchMutationBinding>
    for CapabilityTouchHandler
{
    fn decide(
        &self,
        input: &CapabilityTouchInput,
        _: &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, CapabilityTouchMutationBinding>,
    ) -> HandlerResult<CapabilityTouchInput, CapabilityTouchInput> {
        HandlerResult::Completed(input.clone())
    }

    fn build_candidate(
        &self,
        _: &CapabilityTouchInput,
        decision: CapabilityTouchInput,
        _: &mut CandidateWriter<'_, IdentityExecutionSchema, CapabilityTouchMutationBinding>,
    ) -> HandlerResult<CapabilityTouchInput, CapabilityTouchInput> {
        HandlerResult::Completed(decision)
    }
}
