use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation, WorthQueryTemporalIntentCandidate,
};

use super::super::operation_invocation::{
    WorthQueryTemporalOperationExecution, WorthQueryTemporalOperationInvoker,
};
use super::super::reconstruction_authority::{
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
};
use super::{
    temporal_idempotency, WorthQueryTemporalReentryDenial, WorthQueryTemporalReentryOutcome,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[allow(clippy::too_many_arguments)]
pub(super) fn try_reentry<
    Schema,
    Operation,
    Input,
    Scope,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    ScopeField,
    ScopeValue,
    PrincipalSource,
    QueryAuthorization,
    Invoker,
    IntentEntity,
    IdentityField,
    IdentityValue,
    RevisionField,
    RevisionValue,
    LifecycleField,
    LifecycleValue,
    Authorization,
    Clock,
>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    access: &WorthQueryTemporalReconstructionAccess<
        Schema,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeField,
        ScopeValue,
        PrincipalSource,
        QueryAuthorization,
    >,
    execution: &WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
    >,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
    idempotency: &temporal_idempotency::WorthQueryPreparedTemporalIdempotency,
) -> Result<WorthQueryTemporalReentryOutcome, WorthQueryTemporalReentryDenial>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = Scope,
        Value = ScopeValue,
    >,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = IdentityValue,
    >,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityValue: Clone,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = RevisionValue,
    >,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = RevisionValue,
        > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = LifecycleValue,
    >,
    LifecycleField::Binding:
        worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = LifecycleValue,
        >,
    LifecycleValue: Clone,
    Authorization:
        super::super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    let selected = runtime
        .on_product(request_execution, product.retained_clone())
        .map_err(super::super::WorthQueryConditionalReentryFailure::ProductSelection)?;
    let fresh = access.resolve_fresh_operation_access(request_execution, &selected)?;
    let Some(current) = execution.resolve_current_intent(
        request_execution,
        &selected,
        candidate.record_identity(),
        candidate.revision(),
        &fresh.request,
    )?
    else {
        return Ok(WorthQueryTemporalReentryOutcome::Obsolete);
    };
    let Some(projected) = execution.admit_current_projection(
        request_execution,
        &selected,
        operation,
        candidate,
        &fresh,
        &current,
    )?
    else {
        return Ok(WorthQueryTemporalReentryOutcome::Obsolete);
    };
    Ok(execution.commit_projected_temporal_effect(
        request_execution,
        phase,
        runtime,
        candidate,
        current,
        projected,
        idempotency,
    )?)
}
