use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityDelegationRequest, ApplicationCapabilityRequest,
    },
    application_schema::TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationCapability,
    WorthQueryInstalledApplicationOperation,
};

use super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod binding;
mod replay;
pub(in crate::domain_computation) use binding::{
    WorthQueryDelegationActivationBinding, WorthQueryDelegationActivationEffect,
};
pub(in crate::domain_computation::authorization) mod support;
use support::authorize_target_support;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub fn authorize_capability_delegation<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation:
            worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            >,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>,
    {
        let execution = match phase.execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(_) => return Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase), "phase-custody")),
        };
        self.authorize_capability_delegation_with_request(
            &execution,
            access,
            target,
            operation,
            preconditions,
        )
    }

    pub(in crate::domain_computation) fn authorize_capability_delegation_with_request<
        CommandCapability,
        TargetCapability,
        TargetOperation,
        TargetInput,
        Operation,
        Input,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            CommandCapability,
            Operation,
            Input,
        >,
        target: &WorthQueryInstalledApplicationCapability<
            Schema,
            TargetCapability,
            TargetOperation,
            TargetInput,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, CommandCapability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation:
            worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            >,
        Input: ApplicationCapabilityRequest<Schema, CommandCapability>
            + ApplicationCapabilityDelegationRequest<Schema, TargetCapability>,
    {
        let proposed = access
            .capability_input()
            .delegation_request()
            .map_err(|rejection| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::DelegationRejected,
                    ),
                    rejection.subject(),
                )
            })?;
        let installed = self
            .authorization
            .capability_plan(target)
            .ok_or_else(|| stale(target.contract().name()))?;
        validate_activation_operation(installed, operation)?;
        let observed = authorize_target_support(execution, self, target, &access, &proposed)?;
        let mut access = access;
        let prepared = observed.prepare_activation(
            self,
            installed,
            &proposed,
            target,
            operation,
            &mut access,
        )?;
        let admitted = progress_capability_operation(
            execution,
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::DelegationActivation,
        )?;
        prepared.finish(admitted)
    }
}

fn validate_activation_operation<Schema, Operation, Input>(
    installed: &super::capability_registry::WorthQueryInstalledCapabilityPlan,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let activation = installed
        .delegation()
        .activation
        .as_ref()
        .ok_or_else(|| delegation_denial(installed))?;
    if activation.operation == operation.operation()
        && activation.operation_type == operation.operation()
        && activation.input_type == operation.input_type()
    {
        Ok(())
    } else {
        Err(delegation_denial(installed))
    }
}

fn stale(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleInstalledOperation,
        ),
        subject,
    )
}

fn delegation_denial(
    installed: &super::capability_registry::WorthQueryInstalledCapabilityPlan,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationRejected,
        ),
        installed.contract().name(),
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
