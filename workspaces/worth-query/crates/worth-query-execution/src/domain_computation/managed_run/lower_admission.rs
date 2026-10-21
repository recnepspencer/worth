use std::sync::Arc;

use worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary;
use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeBoundExecutionBasis, BridgeManagedExecutionIntent,
    BridgeManagedExecutionPartialEffectPosture, BridgeManagedExecutionStepContract,
    BridgeManagedExecutionStepLimits, BridgeTruthViewSelector, HistoricalEvaluationDeclaration,
    RuntimeBridge, RuntimeBridgeRelationalSource,
};

use super::WorthQueryManagedRelationalObservation;
use super::WorthQueryManagedTruthReadRequest;

pub(in crate::domain_computation) struct WorthQueryManagedLowerExecutionBasis {
    pub bridge: BridgeBoundExecutionBasis,
    pub relational: WorthQueryManagedRelationalObservation,
    pub product_observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

pub(in crate::domain_computation) struct WorthQueryManagedLowerBinding<'a> {
    operation_identity: &'a str,
    resource_attempt_identity: &'a str,
    resource_envelope: &'a WorthQueryExecutionResourceEnvelope,
}

impl<'a> WorthQueryManagedLowerBinding<'a> {
    pub(in crate::domain_computation) const fn new(
        operation_identity: &'a str,
        resource_attempt_identity: &'a str,
        resource_envelope: &'a WorthQueryExecutionResourceEnvelope,
    ) -> Self {
        Self {
            operation_identity,
            resource_attempt_identity,
            resource_envelope,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryManagedLowerAdmissionFailureKind {
    BridgeSourceProfile,
    RelationalBasis,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    BridgePlanning,
    InstalledStepContract,
    ReadExecution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    BridgeExecutionBasis,
}

pub(in crate::domain_computation) struct WorthQueryManagedLowerAdmissionFailure {
    pub kind: WorthQueryManagedLowerAdmissionFailureKind,
    pub detail: Arc<str>,
}

pub(in crate::domain_computation) fn admit_managed_lower_execution_basis(
    bridge: &RuntimeBridge,
    relational: &RuntimeBridgeRelationalSource,
    binding: WorthQueryManagedLowerBinding<'_>,
    request: WorthQueryManagedTruthReadRequest,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryManagedLowerAdmissionFailure> {
    admit_lower_execution_basis(
        bridge,
        relational,
        binding,
        request,
        WorthQueryExecutionBoundary::BoundedStep,
        execution,
    )
}

pub(in crate::domain_computation) fn admit_atomic_lower_execution_basis(
    bridge: &RuntimeBridge,
    relational: &RuntimeBridgeRelationalSource,
    binding: WorthQueryManagedLowerBinding<'_>,
    request: WorthQueryManagedTruthReadRequest,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryManagedLowerAdmissionFailure> {
    admit_lower_execution_basis(
        bridge,
        relational,
        binding,
        request,
        WorthQueryExecutionBoundary::Atomic,
        execution,
    )
}

fn admit_lower_execution_basis(
    bridge: &RuntimeBridge,
    relational: &RuntimeBridgeRelationalSource,
    binding: WorthQueryManagedLowerBinding<'_>,
    request: WorthQueryManagedTruthReadRequest,
    boundary: WorthQueryExecutionBoundary,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryManagedLowerAdmissionFailure> {
    if binding.resource_envelope.boundary() != boundary {
        return Err(WorthQueryManagedLowerAdmissionFailure {
            kind: WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract,
            detail: Arc::from(
                "installed execution boundary differs from the selected lower admission",
            ),
        });
    }
    let step = match boundary {
        WorthQueryExecutionBoundary::Atomic => None,
        WorthQueryExecutionBoundary::BoundedStep => {
            Some(lower_installed_step_contract(binding.resource_envelope)?)
        }
    };
    admit_native_execution_basis(
        bridge,
        relational,
        binding.operation_identity,
        binding.resource_attempt_identity,
        step,
        request,
        execution,
    )
}

pub(in crate::domain_computation) fn admit_live_lower_execution_basis(
    bridge: &RuntimeBridge,
    relational: &RuntimeBridgeRelationalSource,
    operation: &str,
    attempt: &str,
    buffer_capacity: usize,
    request: WorthQueryManagedTruthReadRequest,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryManagedLowerAdmissionFailure> {
    let capacity =
        u64::try_from(buffer_capacity).map_err(|_| WorthQueryManagedLowerAdmissionFailure {
            kind: WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract,
            detail: Arc::from("live buffer capacity is not representable"),
        })?;
    let step =
        BridgeManagedExecutionStepContract::queue("application-query-live-delivery", capacity)
            .map_err(|detail| WorthQueryManagedLowerAdmissionFailure {
                kind: WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract,
                detail: Arc::from(detail),
            })?;
    admit_native_execution_basis(
        bridge,
        relational,
        operation,
        attempt,
        Some(step),
        request,
        execution,
    )
}

fn admit_native_execution_basis(
    bridge: &RuntimeBridge,
    relational: &RuntimeBridgeRelationalSource,
    operation: &str,
    attempt: &str,
    step: Option<BridgeManagedExecutionStepContract>,
    request: WorthQueryManagedTruthReadRequest,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryManagedLowerExecutionBasis, WorthQueryManagedLowerAdmissionFailure> {
    let expected_source = relational.authoritative_source_profile();
    if bridge.authoritative_source_profile() != Some(&expected_source) {
        return Err(WorthQueryManagedLowerAdmissionFailure {
            kind: WorthQueryManagedLowerAdmissionFailureKind::BridgeSourceProfile,
            detail: Arc::from(
                "Bridge runtime and Relational execution source do not share one authoritative adapter",
            ),
        });
    }
    let (descriptor, packet, replay, diagnostics, delivery, product_observation) =
        request.into_parts();
    let branch = worth_runtime_bridge::facade::TruthBranchIdentity::from_relational_branch_id(
        descriptor.branch_id().0.clone(),
    );
    let relational_basis = relational
        .readmit_branch_basis(&descriptor)
        .map_err(relational_basis_failure)?;
    let current_at_admission = relational
        .observe_branch_basis(relational_basis.identity())
        .map_err(relational_basis_failure)?
        .0
        == descriptor;
    let relational_basis = WorthQueryManagedRelationalObservation::retain(
        relational,
        relational_basis,
        current_at_admission,
    )
    .map_err(relational_basis_failure)?
    .with_product_owner(product_observation.as_ref());
    let snapshot = relational_basis.identity().snapshot_identity().clone();
    let declaration = HistoricalEvaluationDeclaration::new(
        BridgeTruthViewSelector::branch_snapshot(branch.clone(), snapshot.clone()),
        replay,
        diagnostics,
        delivery,
    );
    let planned = bridge
        .plan_truth_view_packet(declaration, packet, execution)
        .map_err(|failure| WorthQueryManagedLowerAdmissionFailure {
            kind: WorthQueryManagedLowerAdmissionFailureKind::BridgePlanning,
            detail: Arc::from(format!("{failure:?}")),
        })?;
    let intent = BridgeManagedExecutionIntent::new(operation, attempt);
    let truth_basis = BridgeAsyncRequestTruthViewBasis::branch_head(branch, snapshot);
    let bridge_basis = match step {
        None => bridge.admit_atomic_execution_basis(intent, truth_basis, planned, execution),
        Some(step) => {
            bridge.admit_managed_execution_basis(intent, step, truth_basis, planned, execution)
        }
    }
    .map_err(|denial| WorthQueryManagedLowerAdmissionFailure {
        kind: match denial.kind() {
            worth_runtime_bridge::facade::BridgeExecutionBasisDenialKind::ExecutionDenied(
                cause,
            ) => WorthQueryManagedLowerAdmissionFailureKind::ReadExecution(
                super::bridge_read_stop::stopped(cause),
            ),
            _ => WorthQueryManagedLowerAdmissionFailureKind::BridgeExecutionBasis,
        },
        detail: Arc::from(denial.detail()),
    })?;
    Ok(WorthQueryManagedLowerExecutionBasis {
        bridge: bridge_basis,
        relational: relational_basis,
        product_observation,
    })
}

fn relational_basis_failure(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryManagedLowerAdmissionFailure {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            WorthQueryManagedLowerAdmissionFailureKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            WorthQueryManagedLowerAdmissionFailureKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            WorthQueryManagedLowerAdmissionFailureKind::SnapshotIdentityExhausted
        }
        _ => WorthQueryManagedLowerAdmissionFailureKind::RelationalBasis,
    };
    WorthQueryManagedLowerAdmissionFailure {
        kind,
        detail: Arc::from(format!("{denial:?}")),
    }
}

fn lower_installed_step_contract(
    resource_envelope: &WorthQueryExecutionResourceEnvelope,
) -> Result<BridgeManagedExecutionStepContract, WorthQueryManagedLowerAdmissionFailure> {
    let installed = resource_envelope
        .bounded_step_contract()
        .map_err(|detail| WorthQueryManagedLowerAdmissionFailure {
            kind: WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract,
            detail: Arc::from(detail),
        })?;
    let limits = BridgeManagedExecutionStepLimits::new(
        installed.max_work_units_per_step(),
        installed.queue_depth_ceiling(),
        installed.chunk_width_ceiling(),
    )
    .with_memory_ceilings(
        installed.scratch_bytes_ceiling(),
        installed.retained_bytes_ceiling(),
    )
    .with_deadline_nanos(installed.deadline_nanos());
    let partial_effects = if installed.partial_effects_may_remain() {
        BridgeManagedExecutionPartialEffectPosture::MayRemain
    } else {
        BridgeManagedExecutionPartialEffectPosture::None
    };
    BridgeManagedExecutionStepContract::new(
        installed.safe_point_family().as_str(),
        limits,
        partial_effects,
    )
    .map_err(|detail| WorthQueryManagedLowerAdmissionFailure {
        kind: WorthQueryManagedLowerAdmissionFailureKind::InstalledStepContract,
        detail: Arc::from(detail),
    })
}

#[cfg(test)]
mod atomic_tests;
