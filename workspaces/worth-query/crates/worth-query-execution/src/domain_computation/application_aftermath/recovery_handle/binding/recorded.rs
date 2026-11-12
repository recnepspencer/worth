//! A recorded binding consumes admitted original sources and fresh operation meaning.
use super::*;
impl WorthQueryRecoveryHandleBinding {
    pub(in crate::domain_computation) fn from_readmitted<
        Schema: worth_query_installation::facade::ApplicationSchema,
        Operation,
        Input,
        Scope,
    >(
        runtime_instance_id: u64,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        outbox: Option<
            crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxBinding,
        >,
        expires_at_unix_ms: u64,
        recorded_provider_observation: Option<crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation>,
    ) -> Result<Self, WorthQueryRecoveryHandleDenial> {
        let aftermath = admission
            .allowed_graph_contract()
            .aftermath()
            .ok_or_else(|| {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                )
            })?;
        match aftermath.recovery() {
            InstalledAftermathRecoveryContract::Admissible { .. } => {}
            InstalledAftermathRecoveryContract::NotAdmitted => {
                return Err(WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::RecoveryNotAdmitted,
                ))
            }
        }
        let principal_scope = admission.operation_scope_binding().clone();
        Ok(Self {
            runtime_instance_id,
            schema_identity: *principal_scope.binding_identity().schema_identity().bytes(),
            commit: publication.relational_commit().clone(),
            committed_product_publication: publication,
            application_binding_generation: principal_scope.binding_identity().generation(),
            installed_operation: *admission.installed_operation_identity_bytes(),
            input_custody: RecoveryInputCustody::Recorded {
                governed_input_identity: admission.governed_input_identity().copied(),
            },
            principal_scope,
            idempotency,
            provider_posture: None,
            recorded_provider_observation,
            committed_dispatch_outbox: outbox,
            installed_aftermath: aftermath.clone(),
            expires_at_unix_ms: Some(expires_at_unix_ms),
        })
    }
}
