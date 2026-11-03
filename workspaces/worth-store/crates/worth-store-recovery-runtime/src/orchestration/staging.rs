use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryOutcome, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingDenial, PhysicalRecoveryStagingSettlementLedger,
};
use crate::progression::{StagedPhysicalRecovery, StagedRecoveryState};

mod command;
mod execution;

mod input_storage;
pub(crate) use input_storage::{
    RecoveryStagingInput, RecoveryStagingInputBacking, RecoveryStagingInputState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryStagingCancellation {
    None,
    AfterSettledCommands(u64),
    Invalid,
}

pub(crate) fn stage_recovery(
    mut input: RecoveryStagingInput,
) -> Result<StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
    match execution::run(&mut input) {
        Ok(execution) => complete(input, execution),
        Err(execution) => Err(block(input, execution)),
    }
}

fn complete(
    input: RecoveryStagingInput,
    execution: execution::StagingExecution,
) -> Result<StagedPhysicalRecovery, PhysicalRecoveryOutcome> {
    if input
        .state()
        .coordination
        .pause_at(
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
        )
        .is_interrupted()
    {
        return Err(block(input, execution));
    }
    if input
        .state()
        .coordination
        .pause_at(
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
        )
        .is_interrupted()
    {
        return Err(block(input, execution));
    }
    let input = input.into_state();
    let base = input.staging.into_base_image();
    Ok(input.staged_backing.finish(StagedRecoveryState {
        reopened_backing: input.reopened_backing,
        publication_backing: input.publication_backing,
        namespace_backing: input.namespace_backing,
        authority: input.authority,
        coordination: input.coordination,
        selection: input.selection,
        custody: input.custody,
        verified_selected_tier_custody: input.verified_selected_tier_custody,
        discovery_counters: input.discovery_counters,
        root_protocol_denials: input.root_protocol_denials,
        integrity: input.integrity,
        freshness: input.freshness,
        fates: input.fates,
        planning_counters: input.planning_counters,
        root_protocol_counters: input.root_protocol_counters,
        base,
        publication: input.publication,
        quiescence: input.quiescence,
        closed: execution.closed.expect("successful execution is closed"),
        staging_counters: execution.counters,
        staging_settlements: execution.settlements,
        integrity_trace: input.integrity_trace,
    }))
}

fn block(
    input: RecoveryStagingInput,
    execution: execution::StagingExecution,
) -> PhysicalRecoveryOutcome {
    let input = input.into_state();
    let store = input.authority.media.store_identity();
    let session_identity = input.authority.session.identity();
    let recovery_effects = input.authority.media.recovery_effect_count();
    assert!(input.coordination.shutdown_is_quiescent());
    let AdmittedPlatformAuthority { media, session, .. } = input.authority;
    drop(media);
    session.block();
    PhysicalRecoveryOutcome::Blocked(Box::new(PhysicalRecoveryBlock::new(
        crate::entry::PhysicalRecoveryBlockCause::Damage(PhysicalRecoveryBlockKind::Staging),
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
    )))
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
