use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::{PhysicalSourceSelection, RecoveryPlanningCounters};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoverySourceDenial,
    PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingDenial,
    PhysicalRecoveryStagingSettlementLedger,
};
use crate::handoff::RecoveryOperationFateSet;
use crate::progression::{
    PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence, RecoveryPublicationPlan,
    RecoveryQuiescencePlan, RecoveryStagingLayoutPlan, StagedPhysicalRecovery,
};

use super::RecoveryCoordination;

mod command;
mod execution;

pub(crate) struct RecoveryStagingInput {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) coordination: RecoveryCoordination,
    pub(crate) selection: PhysicalSourceSelection,
    pub(crate) discovery_counters: PhysicalRecoveryDiscoveryCounters,
    pub(crate) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(crate) integrity: RecoveryIntegrityEvidence,
    pub(crate) freshness: StoreRecoveryBindingFreshnessSample,
    pub(crate) fates: RecoveryOperationFateSet,
    pub(crate) planning_counters: RecoveryPlanningCounters,
    pub(crate) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(crate) staging: RecoveryStagingLayoutPlan,
    pub(crate) publication: RecoveryPublicationPlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) cancellation: RecoveryStagingCancellation,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryStagingCancellation {
    None,
    AfterSettledCommands(u64),
    Invalid,
}

pub(crate) fn stage_recovery(
    input: RecoveryStagingInput,
) -> Result<StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
    match execution::run(&input) {
        Ok(execution) => complete(input, execution),
        Err(execution) => Err(block(input, execution)),
    }
}

fn complete(
    input: RecoveryStagingInput,
    execution: execution::StagingExecution,
) -> Result<StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
    if input
        .coordination
        .pause_at(
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
        )
        .is_interrupted()
    {
        return Err(block(input, execution));
    }
    if input
        .coordination
        .pause_at(
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
        )
        .is_interrupted()
    {
        return Err(block(input, execution));
    }
    let base = input.staging.into_base_image();
    Ok(StagedPhysicalRecovery::new(
        input.authority,
        input.coordination,
        input.selection,
        input.discovery_counters,
        input.root_protocol_denials,
        input.integrity,
        input.freshness,
        input.fates,
        input.planning_counters,
        input.root_protocol_counters,
        base,
        input.publication,
        input.quiescence,
        execution.closed.expect("successful execution is closed"),
        execution.counters,
        execution.settlements,
        input.integrity_trace,
    ))
}

fn block(
    input: RecoveryStagingInput,
    execution: execution::StagingExecution,
) -> PhysicalRecoveryOutcome {
    let store = input.authority.media.store_identity();
    let session_identity = input.authority.session.identity();
    let recovery_effects = input.authority.media.recovery_effect_count();
    assert!(input.coordination.shutdown_is_quiescent());
    let AdmittedPlatformAuthority { media, session, .. } = input.authority;
    drop(media);
    session.block();
    PhysicalRecoveryOutcome::Blocked(PhysicalRecoveryBlock::new(
        PhysicalRecoveryBlockKind::Staging,
        store,
        session_identity,
        PhysicalRecoveryBlockEvidence {
            counters: input.discovery_counters,
            source_denials: input.root_protocol_denials,
            integrity_observations: input.integrity.into_observations(),
            planning_counters: Some(input.planning_counters),
            root_protocol_counters: Some(input.root_protocol_counters),
            staging_counters: Some(execution.counters),
            staging_denial: execution.denial,
            staging_settlements: Some(execution.settlements),
            integrity_trace: input.integrity_trace,
            ..PhysicalRecoveryBlockEvidence::default()
        },
        recovery_effects,
    ))
}

fn empty_execution(
    planned: u64,
    denial: PhysicalRecoveryStagingDenial,
) -> execution::StagingExecution {
    execution::StagingExecution {
        counters: PhysicalRecoveryStagingCounters {
            planned_scheduler_commands: planned,
            ..PhysicalRecoveryStagingCounters::default()
        },
        settlements: PhysicalRecoveryStagingSettlementLedger::new(Vec::new()),
        closed: None,
        denial: Some(denial),
    }
}
