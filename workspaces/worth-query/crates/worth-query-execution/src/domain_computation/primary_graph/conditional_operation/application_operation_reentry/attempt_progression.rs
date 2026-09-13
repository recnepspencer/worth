use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, OperationReads, OperationWrites,
    WorthQueryInstalledApplicationOperation, WorthQueryTemporalIntentCandidate, WritableCapability,
    WritePosture,
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
    ScopeAspect,
    ScopeField,
    ScopeValue,
    ScopeWrite,
    ScopeUnit,
    PrincipalSource,
    QueryAuthorization,
    Invoker,
    IntentEntity,
    IdentityAspect,
    IdentityField,
    IdentityValue,
    IdentityWrite,
    IdentityUnit,
    RevisionAspect,
    RevisionField,
    RevisionValue,
    RevisionWrite,
    RevisionEquality,
    RevisionUnit,
    LifecycleAspect,
    LifecycleField,
    LifecycleValue,
    LifecycleWrite,
    LifecycleEquality,
    LifecycleUnit,
    Authorization,
    Clock,
>(
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
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
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
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    >,
    candidate: &WorthQueryTemporalIntentCandidate<Clock, Input>,
    idempotency: &temporal_idempotency::WorthQueryPreparedTemporalIdempotency,
) -> Result<WorthQueryTemporalReentryOutcome, WorthQueryTemporalReentryDenial>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeValue: Clone,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: OperationReads<Operation>,
    IdentityField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityValue: Clone,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField: OperationReads<Operation> + OperationWrites<Operation>,
    RevisionField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = RevisionValue,
        > + worth_query_installation::facade::WorthQueryTemporalIntentRevisionValue,
    RevisionValue: Clone,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationReads<Operation> + OperationWrites<Operation>,
    LifecycleField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding:
        worth_query_installation::facade::ApplicationReadableScalarValueBinding<
            Value = LifecycleValue,
        >,
    LifecycleValue: Clone,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
    Authorization:
        super::super::WorthQueryTemporalOperationAuthorization<Schema, Operation, Input, Scope>,
{
    let selected = runtime
        .on_product(product.retained_clone())
        .map_err(|denial| format!("temporal selected product denied: {denial:?}"))?;
    let fresh = access.resolve_fresh_operation_access(&selected)?;
    let Some(current) = execution.resolve_current_intent(
        &selected,
        candidate.record_identity(),
        candidate.revision(),
        &fresh.request,
    )?
    else {
        return Ok(WorthQueryTemporalReentryOutcome::Obsolete);
    };
    let Some(projected) =
        execution.admit_current_projection(&selected, operation, candidate, &fresh, &current)?
    else {
        return Ok(WorthQueryTemporalReentryOutcome::Obsolete);
    };
    Ok(execution.commit_projected_temporal_effect(
        runtime,
        candidate,
        current,
        projected,
        idempotency,
    )?)
}
