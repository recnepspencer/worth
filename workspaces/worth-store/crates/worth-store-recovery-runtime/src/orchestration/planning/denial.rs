use worth_store::physical_runtime::StoreRecoveryBindingSampleDenial;
use worth_store_recovery_physics::{
    PhysicalRedoPlanningDenial, RecoveryPlanCost, RecoveryPlanCostDenial, RecoveryPlanLimits,
    RecoveryPlanningCounters,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockCause, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimitFailure, PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoverySourceDenial,
};
use crate::handoff::block_unsupported_scope;
use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::super::recovery_budget::RecoveryAllowance;
use super::super::RecoveryCoordination;

pub(super) fn redo_block(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    counters: PhysicalRecoveryDiscoveryCounters,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    limit: Option<PhysicalRecoveryLimitFailure>,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_with_root_protocol_counters(
        authority,
        coordination,
        PhysicalRecoveryBlockKind::RedoPlanning,
        counters,
        planning_counters,
        root_protocol_counters,
        "canonical-redo-plan",
        limit,
        None,
        source_denials,
    )
}

pub(super) fn redo_denial_block(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    counters: PhysicalRecoveryDiscoveryCounters,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    limit: Option<PhysicalRecoveryLimitFailure>,
    denial: PhysicalRedoPlanningDenial,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_with_root_protocol_counters(
        authority,
        coordination,
        PhysicalRecoveryBlockKind::RedoPlanning,
        counters,
        planning_counters,
        root_protocol_counters,
        "canonical-redo-plan",
        limit,
        Some(PhysicalRecoveryPlanningDenial::Redo(denial)),
        source_denials,
    )
}

pub(super) fn cost_denial_block(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    counters: PhysicalRecoveryDiscoveryCounters,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    denial: RecoveryPlanCostDenial,
    limit: Option<PhysicalRecoveryLimitFailure>,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_with_root_protocol_counters(
        authority,
        coordination,
        PhysicalRecoveryBlockKind::RedoPlanning,
        counters,
        planning_counters,
        root_protocol_counters,
        "recovery-plan-cost",
        limit,
        Some(PhysicalRecoveryPlanningDenial::Cost(denial)),
        source_denials,
    )
}

/// The plan limit physics found the cost past, read beside recovery's
/// declared allowance for it: physics counted against the plan limits it was
/// handed. `None` where those counts do not cross the declared allowance: no
/// limit can state that.
pub(super) fn plan_cost_limit(
    declared: &PhysicalRecoveryLimitDeclaration,
    denial: RecoveryPlanCostDenial,
    limits: RecoveryPlanLimits,
    cost: RecoveryPlanCost,
) -> Option<PhysicalRecoveryLimitFailure> {
    let (dimension, observed, admitted) = match denial {
        RecoveryPlanCostDenial::RedoTargets => (
            PhysicalRecoveryLimitDimension::RedoTargets,
            cost.redo_targets(),
            limits.redo_targets(),
        ),
        RecoveryPlanCostDenial::RedoBytes => (
            PhysicalRecoveryLimitDimension::RedoBytes,
            cost.redo_bytes(),
            limits.redo_bytes(),
        ),
        RecoveryPlanCostDenial::DistinctTargets => (
            PhysicalRecoveryLimitDimension::DistinctPagesAndExtents,
            cost.distinct_targets(),
            limits.distinct_targets(),
        ),
        RecoveryPlanCostDenial::OperationBindings => (
            PhysicalRecoveryLimitDimension::OperationBindings,
            cost.operation_bindings(),
            limits.operation_bindings(),
        ),
        RecoveryPlanCostDenial::ObservationBytes => (
            PhysicalRecoveryLimitDimension::ObservationBytes,
            cost.observation_bytes(),
            limits.observation_bytes(),
        ),
        RecoveryPlanCostDenial::StagingBytes => (
            PhysicalRecoveryLimitDimension::StagingBytes,
            cost.staging_bytes(),
            limits.staging_bytes(),
        ),
        RecoveryPlanCostDenial::RecoveryMemoryBytes => (
            PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            cost.peak_recovery_bytes(),
            limits.recovery_memory_bytes(),
        ),
        RecoveryPlanCostDenial::DirtyFrames => (
            PhysicalRecoveryLimitDimension::DirtyFrames,
            cost.dirty_frames(),
            limits.dirty_frames(),
        ),
    };
    beside(declared, dimension, observed, admitted)
}

/// `observed` past the `admitted` another owner was handed, read beside the
/// whole of `dimension` recovery declared.
fn beside(
    declared: &PhysicalRecoveryLimitDeclaration,
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> Option<PhysicalRecoveryLimitFailure> {
    RecoveryAllowance::declared(declared, dimension)
        .beside(observed, admitted)
        .map(Into::into)
}

pub(super) fn block(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    kind: PhysicalRecoveryBlockKind,
    counters: PhysicalRecoveryDiscoveryCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    artifact: &str,
    limit: Option<PhysicalRecoveryLimitFailure>,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_unsupported_scope(
        authority,
        coordination,
        PhysicalRecoveryBlockCause::of(kind, limit),
        PhysicalRecoveryBlockEvidence {
            counters,
            planning_counters: Some(RecoveryPlanningCounters::default()),
            root_protocol_counters: Some(root_protocol_counters),
            artifact: Some(artifact.to_owned()),
            source_denials,
            ..Default::default()
        },
    )
}

pub(super) fn block_with_planning_attempt_denial(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    kind: PhysicalRecoveryBlockKind,
    counters: PhysicalRecoveryDiscoveryCounters,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    artifact: &str,
    limit: Option<PhysicalRecoveryLimitFailure>,
    planning_denial: PhysicalRecoveryPlanningDenial,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_with_root_protocol_counters(
        authority,
        coordination,
        kind,
        counters,
        planning_counters,
        root_protocol_counters,
        artifact,
        limit,
        Some(planning_denial),
        source_denials,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn block_with_root_protocol_counters(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    kind: PhysicalRecoveryBlockKind,
    counters: PhysicalRecoveryDiscoveryCounters,
    planning_counters: RecoveryPlanningCounters,
    root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    artifact: &str,
    limit: Option<PhysicalRecoveryLimitFailure>,
    planning_denial: Option<PhysicalRecoveryPlanningDenial>,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
) -> PhysicalRecoveryOutcome {
    block_unsupported_scope(
        authority,
        coordination,
        PhysicalRecoveryBlockCause::of(kind, limit),
        PhysicalRecoveryBlockEvidence {
            counters,
            planning_counters: Some(planning_counters),
            root_protocol_counters: Some(root_protocol_counters),
            artifact: Some(artifact.to_owned()),
            planning_denial,
            source_denials,
            ..Default::default()
        },
    )
}

/// The limit a binding sample ran out of. The sampler was handed recovery's
/// declared bindings and redo bytes, so its counts read beside them; its
/// memory refusal is recovery memory's.
pub(super) fn sample_limit(
    failure: &worth_store::physical_runtime::StoreRecoveryBindingSampleFailure,
    declared: &PhysicalRecoveryLimitDeclaration,
) -> Option<PhysicalRecoveryLimitFailure> {
    match failure.denial() {
        StoreRecoveryBindingSampleDenial::OperationBindingLimit => beside(
            declared,
            PhysicalRecoveryLimitDimension::OperationBindings,
            failure.operation_bindings_observed(),
            declared.operation_bindings,
        ),
        StoreRecoveryBindingSampleDenial::RedoByteLimit => beside(
            declared,
            PhysicalRecoveryLimitDimension::RedoBytes,
            failure.redo_bytes_observed(),
            declared.redo_bytes,
        ),
        StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit => {
            super::super::source_memory_limit(declared, failure.allocation_denial()?)
        }
        StoreRecoveryBindingSampleDenial::FreshnessMediaMismatch
        | StoreRecoveryBindingSampleDenial::ForeignCheckpoint
        | StoreRecoveryBindingSampleDenial::MissingCheckpointSecurityBinding
        | StoreRecoveryBindingSampleDenial::InvalidCheckpointSecurityBinding
        | StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding
        | StoreRecoveryBindingSampleDenial::InvalidWalMember
        | StoreRecoveryBindingSampleDenial::ConflictingOperationEvidence => None,
    }
}
