//! Delegation progression consumes one access-bound exact support observation.

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityDelegationRequest, ApplicationCapabilityRequest,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperation,
};

pub(in crate::domain_computation::authorization) use super::super::operation_progression::WorthQueryDelegationResolvedRequest;
use super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRetainedCapabilitySupport,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(in crate::domain_computation::authorization) struct WorthQueryObservedDelegationSupport {
    resolved: WorthQueryDelegationResolvedRequest,
    supporting: WorthQueryRetainedCapabilitySupport,
}

impl WorthQueryObservedDelegationSupport {
    pub(in crate::domain_computation::authorization) const fn new(
        resolved: WorthQueryDelegationResolvedRequest,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Self {
        Self {
            resolved,
            supporting,
        }
    }

    pub(super) fn prepare_activation<
        Schema,
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
        Scope,
        Context,
    >(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        installed: &super::super::capability_registry::WorthQueryInstalledCapabilityPlan,
        proposed: &worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequestProjection<Schema, Scope, Context>,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        access: &mut WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
    ) -> Result<
        super::binding::WorthQueryPreparedDelegationActivation,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>,
    {
        let prepared = super::binding::bind_activation(
            runtime,
            installed,
            proposed,
            self.resolved,
            target,
            operation,
        )?;
        access
            .retain_observed_support(self.supporting)
            .map_err(|()| {
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    access.operation(),
                )
            })?;
        Ok(prepared)
    }
}

pub(super) fn authorize_target_support<
    Schema,
    CommandCapability,
    TargetCapability,
    TargetOperation,
    TargetInput,
    Operation,
    Input,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    target_capability: &WorthQueryInstalledApplicationCapability<
        Schema,
        TargetCapability,
        TargetOperation,
        TargetInput,
    >,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        CommandCapability,
        Operation,
        Input,
    >,
    proposed: &worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRequestProjection<
        Schema,
        <Input as ApplicationCapabilityDelegationRequest<Schema, TargetCapability>>::Scope,
        <Input as ApplicationCapabilityDelegationRequest<Schema, TargetCapability>>::Context,
    >,
) -> Result<WorthQueryObservedDelegationSupport, WorthQueryOperationAuthorizationDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, CommandCapability>
        + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>,
{
    access
        .with_exact_observation(runtime, |observation| {
            observation.authorize_delegation_support(target_capability, proposed)
        })?
        .ok_or_else(|| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                access.operation(),
            )
        })?
}
