use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationSchema,
    WorthQueryInstalledApplicationOperation, WorthQueryTemporalIntentCandidate,
};

use super::isolate_invoker;
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::conditional_operation::{
    operation_invocation::{
        WorthQueryCurrentTemporalIntent, WorthQueryTemporalOperationExecution,
        WorthQueryTemporalOperationInvoker,
    },
    reconstruction_authority::WorthQueryFreshTemporalOperationAccess,
    WorthQueryTemporalOperationAuthorization,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionSnapshot, WorthQuerySelectedProductOperation,
};

pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryAdmittedTemporalProjection<
    Schema,
    Operation,
    Input,
    Scope,
    Projection,
> {
    pub(super) admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    pub(super) projection:
        WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
    pub(super) host_projection: Projection,
}

#[rustfmt::skip]
impl<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityField, IdentityValue, RevisionField, RevisionValue, LifecycleField, LifecycleValue, Authorization>
    WorthQueryTemporalOperationExecution<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityField, IdentityValue, RevisionField, RevisionValue, LifecycleField, LifecycleValue, Authorization>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityValue: Clone,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = RevisionValue>,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue> + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<Entity = IntentEntity, Value = LifecycleValue>,
    LifecycleField::Binding: ApplicationReadableScalarValueBinding<Value = LifecycleValue>,
    LifecycleValue: Clone,
    Authorization: WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn admit_current_projection<Principal, PrincipalIdentity, Clock>(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &WorthQuerySelectedProductOperation<'_, Schema>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
        fresh: &WorthQueryFreshTemporalOperationAccess<Schema, Principal, PrincipalIdentity, Scope>,
        current: &WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>,
    ) -> Result<Option<WorthQueryAdmittedTemporalProjection<Schema, Operation, Input, Scope, Invoker::Projection>>, WorthQueryTemporalReentryDenial>
    {
        let preconditions = isolate_invoker(|| self.invoker.preconditions(candidate.input()))
            .map_err(|super::invoker_isolation::TemporalInvokerPanicked| super::super::WorthQueryConditionalReentryFailure::PreconditionInvokerPanicked)?;
        let admission = self
            .authorization
            .authorize(execution,
                product,
                &fresh.principal,
                &fresh.scope,
                operation,
                candidate.input(),
                preconditions,
                &fresh.request,
            )
            .map_err(WorthQueryTemporalReentryDenial::from_authorization)?;
        let projected = isolate_invoker(|| {
            self.invariant.project_admitted_operation_with_request(execution,
                &admission,
                |reader, projected_scope| {
                    let current_intent = self.observe_current_intent(execution,
                        reader,
                        current.identity_value.clone(),
                        &current.expected_revision,
                    );
                    let host_projection = matches!(current_intent, Ok(true)).then(|| {
                        self.invoker
                            .project(candidate.input(), reader, projected_scope)
                    });
                    (host_projection, current_intent)
                }, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        })
        .map_err(|super::invoker_isolation::TemporalInvokerPanicked| super::super::WorthQueryConditionalReentryFailure::ProjectionInvokerPanicked)?
        .map_err(WorthQueryTemporalReentryDenial::from_projection)?;
        let ((host_projection, current_intent), projection, _) = projected.into_parts();
        if !current_intent? {
            return Ok(None);
        }
        let host_projection = host_projection
            .ok_or(super::super::WorthQueryConditionalReentryFailure::IntentObsoleteBeforeProjection)?
            .map_err(super::super::WorthQueryConditionalReentryFailure::Invocation)?;
        Ok(Some(WorthQueryAdmittedTemporalProjection {
            admission,
            projection,
            host_projection,
        }))
    }
}
