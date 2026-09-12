use std::{collections::BTreeMap, num::NonZeroUsize};

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationSchema, EqualityPredicate,
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource, WorthQueryTemporalIntentProjector,
    WritePosture,
};

use super::installation::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use super::reconstruction_authority::{
    WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
};
mod denial;
mod reconciliation;
mod source_record_binding;
pub(super) use denial::{
    bridge_reconstruction_denial, reconstruction_denial, retention_capacity_reconstruction_denial,
    retention_identity_reconstruction_denial, snapshot_capacity_reconstruction_denial,
    snapshot_identity_reconstruction_denial,
};
pub(super) use reconciliation::{reconcile_prepared_temporal_intents, reconcile_temporal_intents};
#[cfg(test)]
mod tests;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use source_record_binding::bind_source_records;
pub(super) use source_record_binding::WorthQueryReconstructedTemporalIntent;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryTemporalReconstructionWork {
    pub(super) examined_candidates: usize,
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) total_work_units: usize,
}

pub(super) struct WorthQueryTemporalReconstruction<Clock, Input> {
    pub(super) intents: BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    pub(super) work: WorthQueryTemporalReconstructionWork,
}

pub(super) fn reconstruct_temporal_intents<
    Schema,
    ApplicationOperation,
    Input,
    D,
    O,
    F,
    Node,
    Provider,
    Clock,
    Source,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Projector,
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
    IntentEntity,
    IdentityAspect,
    IdentityField,
    IdentityValue,
    IdentityWrite,
    IdentityUnit,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &WorthQueryInstalledTemporalConditionalOperation<
        Schema,
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >,
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
    identity_field: ApplicationFieldRef<
        Schema,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        EqualityPredicate,
        IdentityUnit,
    >,
    product: &crate::basis::WorthQueryProductBranchLease,
) -> Result<
    WorthQueryTemporalReconstruction<Clock, Input>,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema,
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
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
    QueryAuthorization: super::WorthQueryTemporalQueryAuthorization<
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    IdentityField:
        worth_query_installation::facade::DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: worth_query_installation::facade::ApplicationReadableScalarValueBinding<
        Value = IdentityValue,
    >,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
{
    let admission = isolate_principal_source(access)?;
    let (external, request) = admission.into_parts();
    let selected = runtime
        .on_product(product.retained_clone())
        .map_err(product_denial)?;
    let principal = selected
        .resolve_authenticated_principal(
            &access.principal_binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(denial::principal)?;
    let scope = selected
        .resolve_entity(
            access.scope_field,
            access.scope_value.clone(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(denial::entity)?;
    let query_access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let bounds = binding.bounds();
    let maximum_results = NonZeroUsize::new(bounds.maximum_reconstruction_rows())
        .expect("installed temporal bounds are non-zero");
    let maximum_work = NonZeroUsize::new(bounds.maximum_query_work())
        .expect("installed temporal bounds are non-zero");
    let source_product = selected.product().retained_clone();
    let (_, product, application_basis) = selected.into_parts();
    let controls = WorthQueryApplicationQueryControls::product_one_shot(
        product,
        application_basis,
        maximum_results,
        maximum_work,
        &request,
    );
    let plan = access
        .query_authorization
        .admit(
            runtime,
            binding.query(),
            &query_access,
            binding.parameters().clone(),
            controls,
        )
        .map_err(denial::query_authorization)?;
    let result = runtime
        .execute_application_query_one_shot(plan)
        .map_err(denial::one_shot)?;
    let receipt = result.receipt();
    let work = WorthQueryTemporalReconstructionWork {
        examined_candidates: receipt.examined_candidate_count(),
        projected_records: receipt.projected_record_count(),
        projected_fields: receipt.projected_field_count(),
        total_work_units: receipt.total_work_units(),
    };
    let candidates =
        super::temporal_intent_projection::project_unique_candidates(binding, result.into_rows())?;
    let intents = bind_source_records(
        runtime,
        candidates,
        identity_field,
        &request,
        &source_product,
    )?;
    Ok(WorthQueryTemporalReconstruction { intents, work })
}

pub(super) fn product_denial(
    denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    use crate::basis::WorthQueryProductBranchAdmissionDenial as Kind;
    match denial {
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => snapshot_capacity_reconstruction_denial(maximum_active_snapshots),
        Kind::RetentionCapacityExhausted => retention_capacity_reconstruction_denial(),
        Kind::RetentionIdentityExhausted => retention_identity_reconstruction_denial(),
        Kind::SnapshotIdentityExhausted => snapshot_identity_reconstruction_denial(),
        denial => reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionIntent,
            format!("selected product admission failed: {denial:?}"),
        ),
    }
}

fn isolate_principal_source<
    Schema,
    Binding,
    Mapping,
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
>(
    access: &WorthQueryTemporalReconstructionAccess<
        Schema,
        Binding,
        Mapping,
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
) -> Result<
    super::WorthQueryTemporalPrincipalAdmission<Schema>,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    PrincipalIdentity: 'static,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
{
    match access.fresh_admission() {
        Ok(admission) => Ok(admission),
        Err(failure) => Err(reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionPrincipal,
            format!("{:?}: {}", failure.kind(), failure.detail()),
        )),
    }
}
