use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    TypedApplicationReadableValue, WorthQueryInstalledApplicationOperation,
    WorthQueryTemporalIntentCandidate, WorthQueryTemporalIntentRevisionValue, WritableCapability,
    WritePosture,
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
impl<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
    WorthQueryTemporalOperationExecution<Schema, Operation, Input, Scope, Invoker, IntentEntity, IdentityAspect, IdentityField, IdentityValue, IdentityWrite, IdentityUnit, RevisionAspect, RevisionField, RevisionValue, RevisionWrite, RevisionEquality, RevisionUnit, LifecycleAspect, LifecycleField, LifecycleValue, LifecycleWrite, LifecycleEquality, LifecycleUnit, Authorization>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: OperationReads<Operation>,
    IdentityValue: TypedApplicationReadableValue + Clone,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationReads<Operation> + OperationWrites<Operation>,
    RevisionValue: WorthQueryTemporalIntentRevisionValue + TypedApplicationReadableValue + Clone,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationReads<Operation> + OperationWrites<Operation>,
    LifecycleValue: TypedApplicationReadableValue + Clone,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
    Authorization: WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn admit_current_projection<Principal, PrincipalIdentity, Clock>(
        &self,
        product: &WorthQuerySelectedProductOperation<'_, Schema>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
        fresh: &WorthQueryFreshTemporalOperationAccess<Schema, Principal, PrincipalIdentity, Scope>,
        current: &WorthQueryCurrentTemporalIntent<Schema, IntentEntity, IdentityValue, RevisionValue>,
    ) -> Result<Option<WorthQueryAdmittedTemporalProjection<Schema, Operation, Input, Scope, Invoker::Projection>>, WorthQueryTemporalReentryDenial>
    where
        PrincipalIdentity: worth_query_installation::facade::TypedApplicationIdentityValue,
    {
        let preconditions = isolate_invoker(|| self.invoker.preconditions(candidate.input()))
            .map_err(|detail| format!("temporal operation preconditions failed: {detail}"))?;
        let admission = self
            .authorization
            .authorize(
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
            self.invariant.project_admitted_operation(
                &admission,
                |reader, projected_scope| {
                    let current_intent = self.observe_current_intent(
                        reader,
                        current.identity_value.clone(),
                        &current.expected_revision,
                    );
                    let host_projection = current_intent.is_ok().then(|| {
                        self.invoker
                            .project(candidate.input(), reader, projected_scope)
                    });
                    (host_projection, current_intent)
                },
            )
        })
        .map_err(|detail| format!("temporal operation projection failed: {detail}"))?
        .map_err(WorthQueryTemporalReentryDenial::from_projection)?;
        let ((host_projection, current_intent), projection, _) = projected.into_parts();
        if current_intent.is_err() {
            return Ok(None);
        }
        let host_projection = host_projection
            .ok_or_else(|| "temporal intent became obsolete before host projection".to_string())?
            .map_err(|failure| format!("{:?}: {}", failure.kind(), failure.detail()))?;
        Ok(Some(WorthQueryAdmittedTemporalProjection {
            admission,
            projection,
            host_projection,
        }))
    }
}
