use std::collections::BTreeMap;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, EqualityPredicate,
    WorthQueryTemporalIntentCandidate, WritePosture,
};

use super::{
    reconstruction_denial, WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
};

pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryReconstructedTemporalIntent<
    Clock,
    Input,
> {
    pub(in crate::domain_computation::primary_graph::conditional_operation) lifecycle_token:
        std::sync::Arc<()>,
    candidate: WorthQueryTemporalIntentCandidate<Clock, Input>,
    source_record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
}

impl<Clock, Input> WorthQueryReconstructedTemporalIntent<Clock, Input> {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn candidate(
        &self,
    ) -> &WorthQueryTemporalIntentCandidate<Clock, Input> {
        &self.candidate
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn source_record(
        &self,
    ) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
        self.source_record
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_source_records<
    Schema,
    Clock,
    Input,
    IntentEntity,
    IdentityAspect,
    IdentityField,
    IdentityValue,
    IdentityWrite,
    IdentityUnit,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    candidates: BTreeMap<String, WorthQueryTemporalIntentCandidate<Clock, Input>>,
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
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    product: &crate::basis::WorthQueryProductBranchLease,
) -> Result<
    BTreeMap<String, WorthQueryReconstructedTemporalIntent<Clock, Input>>,
    WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema,
    IdentityField: DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
{
    let selected = runtime
        .on_product(product.retained_clone())
        .map_err(super::product_denial)?;
    candidates
        .into_iter()
        .map(|(identity, candidate)| {
            let value = IdentityField::Binding::decode(candidate.record_identity()).map_err(|_| {
                    reconstruction_denial(
                        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionIntent,
                        format!(
                            "temporal intent `{identity}` projected an invalid record identity"
                        ),
                    )
                })?;
            let record = selected
                .resolve_entity(
                    identity_field,
                    value,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .map_err(|denial| match denial.kind() {
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    } => super::snapshot_capacity_reconstruction_denial(
                        maximum_active_snapshots,
                    ),
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::RetentionCapacityExhausted => {
                        super::retention_capacity_reconstruction_denial()
                    }
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::RetentionIdentityExhausted => {
                        super::retention_identity_reconstruction_denial()
                    }
                    crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::SnapshotIdentityExhausted => {
                        super::snapshot_identity_reconstruction_denial()
                    }
                    _ => reconstruction_denial(
                        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionIntent,
                        format!("{:?}: {}", denial.kind(), denial.subject()),
                    ),
                })?;
            let entity = record.entity_id();
            let source_record =
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                    entity.partition_id.0,
                    entity.local_slot.0,
                    entity.generation.0,
                );
            Ok((
                identity,
                WorthQueryReconstructedTemporalIntent {
                    lifecycle_token: Default::default(),
                    candidate,
                    source_record,
                },
            ))
        })
        .collect()
}
