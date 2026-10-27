use super::super::super::{value_capture::with_scalar, WorthQueryResourceFrameKind};
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;

type ResultValue<Schema, Binding> = <<Binding as ApplicationResourceBinding<Schema>>::ResultBinding as ApplicationScalarValueBinding>::Value;
type FailureValue<Schema, Binding> = <<Binding as ApplicationResourceBinding<Schema>>::FailureBinding as ApplicationScalarValueBinding>::Value;

impl<Schema: ApplicationSchema> WorthQueryApplicationResourcesRequest<'_, '_, '_, Schema> {
    pub fn with_result<Binding: ApplicationResourceBinding<Schema>, R>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        inspect: impl FnOnce(&ResultValue<Schema, Binding>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial> {
        self.inspect(
            selector,
            Purpose::Result,
            |_, _, _| Ok(None),
            |execution, row| {
                let Some(result) = row
                    .occurrence()
                    .result
                    .as_ref()
                    .filter(|value| value.kind == WorthQueryResourceFrameKind::Result)
                else {
                    return Ok(None);
                };
                with_scalar::<Binding::ResultBinding, _>(execution, &result.value.value, inspect)
                    .map(Some)
                    .map_err(ResourceCause::Value)
            },
        )
    }

    pub fn with_result_governed<Binding, Capability, Operation, Input, R>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        inspect: impl FnOnce(&ResultValue<Schema, Binding>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::Result,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |execution, row| {
                let Some(result) = row
                    .occurrence()
                    .result
                    .as_ref()
                    .filter(|value| value.kind == WorthQueryResourceFrameKind::Result)
                else {
                    return Ok(None);
                };
                with_scalar::<Binding::ResultBinding, _>(execution, &result.value.value, inspect)
                    .map(Some)
                    .map_err(ResourceCause::Value)
            },
        )
    }

    pub fn with_domain_failure<Binding: ApplicationResourceBinding<Schema>, R>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        inspect: impl FnOnce(&FailureValue<Schema, Binding>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial> {
        self.inspect(
            selector,
            Purpose::DomainFailure,
            |_, _, _| Ok(None),
            |execution, row| {
                let Some(result) = row
                    .occurrence()
                    .result
                    .as_ref()
                    .filter(|value| value.kind == WorthQueryResourceFrameKind::DomainFailure)
                else {
                    return Ok(None);
                };
                with_scalar::<Binding::FailureBinding, _>(execution, &result.value.value, inspect)
                    .map(Some)
                    .map_err(ResourceCause::Value)
            },
        )
    }

    pub fn with_domain_failure_governed<Binding, Capability, Operation, Input, R>(
        &self,
        selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        inspect: impl FnOnce(&FailureValue<Schema, Binding>) -> R,
    ) -> Result<Option<R>, WorthQueryResourceActionDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = AccessScope<Schema, Binding>>,
    {
        self.inspect(
            selector,
            Purpose::DomainFailure,
            |installed, principal, scope| self.pending(installed, principal, scope, capability),
            |execution, row| {
                let Some(result) = row
                    .occurrence()
                    .result
                    .as_ref()
                    .filter(|value| value.kind == WorthQueryResourceFrameKind::DomainFailure)
                else {
                    return Ok(None);
                };
                with_scalar::<Binding::FailureBinding, _>(execution, &result.value.value, inspect)
                    .map(Some)
                    .map_err(ResourceCause::Value)
            },
        )
    }
}
