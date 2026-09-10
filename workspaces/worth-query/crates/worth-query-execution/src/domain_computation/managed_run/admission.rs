use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;

use super::semantic_basis::{
    validate_managed_semantic_basis, WorthQueryManagedSemanticBasisDenial,
    WorthQueryManagedSemanticBasisObservation,
};
#[cfg(test)]
use super::WorthQueryAdmittedDirectRun;
use super::{
    WorthQueryManagedRunCounters, WorthQueryManagedRunDenial, WorthQueryManagedRunDenialKind,
};
use crate::domain_computation::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime, WorthQueryWorkflowExecutionResourceAttempt,
};

impl WorthQueryExecutionRuntime {
    #[cfg(test)]
    pub(in crate::domain_computation) fn admit_direct_run(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        resource_attempt: WorthQueryDirectExecutionResourceAttempt,
        bridge_basis: BridgeBoundExecutionBasis,
        relational_basis: WorthQueryManagedRelationalObservation,
    ) -> Result<WorthQueryAdmittedDirectRun, WorthQueryManagedRunAdmissionRejection> {
        let counters = match validate_direct_run(
            self,
            operation,
            &resource_attempt,
            &bridge_basis,
            &relational_basis,
        ) {
            Ok(counters) => counters,
            Err(denial) => {
                return Err(WorthQueryManagedRunAdmissionRejection {
                    denial,
                    resource_attempt,
                    bridge_basis,
                    relational_basis,
                });
            }
        };
        Ok(WorthQueryAdmittedDirectRun::new(
            operation,
            resource_attempt,
            bridge_basis,
            relational_basis,
            counters,
        ))
    }
}

#[cfg(test)]
fn validate_direct_run(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt: &WorthQueryDirectExecutionResourceAttempt,
    bridge_basis: &BridgeBoundExecutionBasis,
    relational_basis: &WorthQueryManagedRelationalObservation,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    let counters = validate_direct_run_head(runtime, operation, resource_attempt)?;
    validate_direct_run_lower(
        operation,
        resource_attempt,
        bridge_basis,
        relational_basis,
        None,
        counters,
    )
}

pub(super) fn validate_direct_run_head(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt: &WorthQueryDirectExecutionResourceAttempt,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    let mut counters = WorthQueryManagedRunCounters::default();
    counters.checked_query_runtime();
    validate_query_runtime(runtime, operation, &counters)?;
    counters.checked_resource_attempt();
    validate_resource_attempt(operation, resource_attempt, &counters)?;
    Ok(counters)
}

pub(super) fn validate_direct_run_lower(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt: &WorthQueryDirectExecutionResourceAttempt,
    bridge_basis: &BridgeBoundExecutionBasis,
    relational_basis: &WorthQueryManagedRelationalObservation,
    product_observation: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    counters: WorthQueryManagedRunCounters,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    validate_run_lower(
        operation,
        resource_attempt.attempt_identity().as_str(),
        bridge_basis,
        relational_basis,
        product_observation,
        counters,
    )
}

pub(super) fn validate_workflow_run_head(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt: &WorthQueryWorkflowExecutionResourceAttempt,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    let mut counters = WorthQueryManagedRunCounters::default();
    counters.checked_query_runtime();
    validate_query_runtime(runtime, operation, &counters)?;
    counters.checked_resource_attempt();
    let attempt_operation = resource_attempt.binding_authority();
    if attempt_operation.binding_identity() != operation.binding_identity()
        || resource_attempt.operation_resources().binding_identity() != operation.binding_identity()
        || !super::WorthQueryWorkflowRunAffinity::provider_session_matches_attempt(resource_attempt)
    {
        return Err(denial(
            WorthQueryManagedRunDenialKind::ResourceAttemptMismatch,
            "managed workflow resource attempt does not belong to the exact bound operation",
            &counters,
        ));
    }
    Ok(counters)
}

pub(super) fn validate_workflow_run_lower(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt: &WorthQueryWorkflowExecutionResourceAttempt,
    bridge_basis: &BridgeBoundExecutionBasis,
    relational_basis: &WorthQueryManagedRelationalObservation,
    product_observation: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    counters: WorthQueryManagedRunCounters,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    validate_run_lower(
        operation,
        resource_attempt.attempt_identity().as_str(),
        bridge_basis,
        relational_basis,
        product_observation,
        counters,
    )
}

fn validate_run_lower(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt_identity: &str,
    bridge_basis: &BridgeBoundExecutionBasis,
    relational_basis: &WorthQueryManagedRelationalObservation,
    product_observation: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    mut counters: WorthQueryManagedRunCounters,
) -> Result<WorthQueryManagedRunCounters, WorthQueryManagedRunDenial> {
    counters.checked_bridge_intent();
    validate_bridge_intent(
        operation,
        resource_attempt_identity,
        bridge_basis,
        &counters,
    )?;
    counters.checked_bridge_source();
    validate_source_runtime(bridge_basis, relational_basis, &counters)?;
    counters.checked_relational_basis();
    validate_relational_snapshot(bridge_basis, relational_basis, &counters)?;
    counters.checked_semantic_basis();
    validate_semantic_basis(
        operation,
        bridge_basis,
        relational_basis,
        product_observation,
        &counters,
    )?;
    Ok(counters)
}

#[cfg(test)]
pub(crate) struct WorthQueryManagedRunAdmissionRejection {
    denial: WorthQueryManagedRunDenial,
    resource_attempt: WorthQueryDirectExecutionResourceAttempt,
    bridge_basis: BridgeBoundExecutionBasis,
    relational_basis: WorthQueryManagedRelationalObservation,
}

#[cfg(test)]
impl WorthQueryManagedRunAdmissionRejection {
    pub(crate) fn denial(&self) -> &WorthQueryManagedRunDenial {
        &self.denial
    }

    pub(crate) fn into_resource_attempt(self) -> WorthQueryDirectExecutionResourceAttempt {
        self.resource_attempt
    }
}

#[cfg(test)]
impl std::fmt::Debug for WorthQueryManagedRunAdmissionRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryManagedRunAdmissionRejection")
            .field("denial", &self.denial)
            .field("bridge_basis", &self.bridge_basis.identity())
            .field("relational_basis", &self.relational_basis.identity())
            .finish()
    }
}

fn validate_query_runtime(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryExecutionBoundOperationAuthority,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    if !operation.belongs_to(runtime) {
        return Err(denial(
            WorthQueryManagedRunDenialKind::ForeignQueryRuntime,
            "managed run operation belongs to a different Query execution runtime",
            counters,
        ));
    }
    if !operation.belongs_to_current_installation(runtime) {
        return Err(denial(
            WorthQueryManagedRunDenialKind::StaleInstallationGeneration,
            "managed run operation belongs to a stale installation generation",
            counters,
        ));
    }
    Ok(())
}

fn validate_resource_attempt(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    attempt: &WorthQueryDirectExecutionResourceAttempt,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    let attempt_operation = attempt.binding_authority();
    if attempt_operation.binding_identity() != operation.binding_identity()
        || attempt.resources().binding_identity() != operation.binding_identity()
        || attempt.managed_provider_session().attempt_identity()
            != attempt.attempt_identity().as_str()
    {
        return Err(denial(
            WorthQueryManagedRunDenialKind::ResourceAttemptMismatch,
            "managed run resource attempt does not belong to the exact bound operation",
            counters,
        ));
    }
    Ok(())
}

fn validate_bridge_intent(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    resource_attempt_identity: &str,
    bridge: &BridgeBoundExecutionBasis,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    let intent = bridge.managed_intent();
    if intent.operation_binding_identity() != operation.binding_identity()
        || intent.resource_attempt_identity() != resource_attempt_identity
    {
        return Err(denial(
            WorthQueryManagedRunDenialKind::BridgeManagedIntentMismatch,
            "Bridge execution authority was not minted for this exact operation and resource attempt",
            counters,
        ));
    }
    Ok(())
}

fn validate_source_runtime(
    bridge: &BridgeBoundExecutionBasis,
    relational: &WorthQueryManagedRelationalObservation,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    let profile = bridge.authoritative_source_profile().ok_or_else(|| {
        denial(
            WorthQueryManagedRunDenialKind::MissingBridgeSourceAuthority,
            "managed Relational run requires a Bridge authoritative source profile",
            counters,
        )
    })?;
    if profile.runtime_instance_id() != relational.identity().runtime_instance_id() {
        return Err(denial(
            WorthQueryManagedRunDenialKind::ForeignRelationalRuntime,
            "Bridge source and Relational execution lease belong to different runtimes",
            counters,
        ));
    }
    Ok(())
}

fn validate_relational_snapshot(
    bridge: &BridgeBoundExecutionBasis,
    relational: &WorthQueryManagedRelationalObservation,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    if bridge.observation().snapshot_identity() != relational.identity().snapshot_identity() {
        return Err(denial(
            WorthQueryManagedRunDenialKind::RelationalSnapshotMismatch,
            "Bridge truth observation and Relational execution lease name different snapshots",
            counters,
        ));
    }
    Ok(())
}

fn validate_semantic_basis(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    bridge: &BridgeBoundExecutionBasis,
    relational: &WorthQueryManagedRelationalObservation,
    product_observation: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    counters: &WorthQueryManagedRunCounters,
) -> Result<(), WorthQueryManagedRunDenial> {
    let observation = WorthQueryManagedSemanticBasisObservation {
        semantic: operation.semantic_basis(),
        bridge_kind: bridge.request().basis_binding().truth_view_basis().kind(),
        bridge_authority_basis_digest: bridge.observation().authority_basis().digest(),
        relational_current_at_admission: relational.was_current_at_admission(),
        product_observation,
    };
    match validate_managed_semantic_basis(observation) {
        Ok(()) => Ok(()),
        Err(WorthQueryManagedSemanticBasisDenial::Mismatch) => Err(denial(
            WorthQueryManagedRunDenialKind::SemanticBasisMismatch,
            "managed run lower basis does not match the admitted semantic basis",
            counters,
        )),
        Err(WorthQueryManagedSemanticBasisDenial::Unsupported) => Err(denial(
            WorthQueryManagedRunDenialKind::SemanticBasisUnsupported,
            "this semantic basis cannot authorize an ordinary managed Relational run",
            counters,
        )),
    }
}

fn denial(
    kind: WorthQueryManagedRunDenialKind,
    detail: &'static str,
    counters: &WorthQueryManagedRunCounters,
) -> WorthQueryManagedRunDenial {
    WorthQueryManagedRunDenial::new(kind, detail, counters.clone())
}
