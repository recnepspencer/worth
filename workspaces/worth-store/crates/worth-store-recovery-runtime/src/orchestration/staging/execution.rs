use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    PhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommandOutcome,
};

use crate::entry::{
    PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingDenial,
    PhysicalRecoveryStagingSettlement, PhysicalRecoveryStagingSettlementLedger,
};
use crate::progression::ClosedRecoveryStagingGeneration;

use super::RecoveryStagingInput;

mod counters;

pub(super) struct StagingExecution {
    pub(super) counters: PhysicalRecoveryStagingCounters,
    pub(super) settlements: PhysicalRecoveryStagingSettlementLedger,
    pub(super) closed: Option<ClosedRecoveryStagingGeneration>,
    pub(super) denial: Option<PhysicalRecoveryStagingDenial>,
}

struct ExecutionProgress {
    counters: PhysicalRecoveryStagingCounters,
    settlements: Vec<PhysicalRecoveryStagingSettlement>,
    content: Sha256,
    media_handle_baseline: (u64, u64),
}

impl ExecutionProgress {
    fn new(input: &RecoveryStagingInput) -> Self {
        Self {
            counters: PhysicalRecoveryStagingCounters {
                planned_scheduler_commands: input.quiescence.staging_commands(),
                ..PhysicalRecoveryStagingCounters::default()
            },
            settlements: Vec::with_capacity(input.staging.commands().len()),
            content: Sha256::new(),
            media_handle_baseline: {
                let observed = input.authority.media.handle_observation();
                (
                    observed.live_file_handles(),
                    observed.live_directory_handles(),
                )
            },
        }
    }
}

pub(super) fn run(input: &RecoveryStagingInput) -> Result<StagingExecution, StagingExecution> {
    if !super::command::exact_plan_commands(input)
        || input.cancellation == super::RecoveryStagingCancellation::Invalid
    {
        return Err(super::empty_execution(
            input.quiescence.staging_commands(),
            PhysicalRecoveryStagingDenial::InvalidPlan,
        ));
    }
    let mut progress = ExecutionProgress::new(input);
    if let Err(mut execution) = execute_commands(input, &mut progress) {
        record_quiescence(
            input,
            &mut execution.counters,
            progress.media_handle_baseline,
        );
        return Err(execution);
    }
    close_staging(input, progress)
}

fn execute_commands(
    input: &RecoveryStagingInput,
    progress: &mut ExecutionProgress,
) -> Result<(), StagingExecution> {
    for command in input.staging.commands() {
        let Some(declaration) = PhysicalRecoveryStagingCommand::new(
            command.ordinal(),
            input.publication.plan_identity(),
            input.staging.staging_generation(),
            command.artifact(),
            command.bytes(),
            command.payload_digest(),
        ) else {
            return Err(failed(
                progress.counters,
                std::mem::take(&mut progress.settlements),
                PhysicalRecoveryStagingDenial::InvalidPlan,
            ));
        };
        let outcome = input
            .coordination
            .owner()
            .execute_staging_command(&input.authority.media, declaration);
        record_command_outcome(progress, command.ordinal(), outcome)?;
        if matches!(
            input.cancellation,
            super::RecoveryStagingCancellation::AfterSettledCommands(settled)
                if settled == progress.settlements.len() as u64
        ) {
            let settled_commands = progress.counters.commands_settled;
            return Err(failed(
                progress.counters,
                std::mem::take(&mut progress.settlements),
                PhysicalRecoveryStagingDenial::CancelledAfterPartialStaging { settled_commands },
            ));
        }
    }
    Ok(())
}

fn record_command_outcome(
    progress: &mut ExecutionProgress,
    ordinal: u64,
    outcome: PhysicalRecoveryStagingCommandOutcome,
) -> Result<(), StagingExecution> {
    match outcome {
        PhysicalRecoveryStagingCommandOutcome::Completed(completed) => {
            counters::record_completed(&mut progress.counters, &mut progress.content, &completed);
            progress
                .settlements
                .push(PhysicalRecoveryStagingSettlement::Completed(completed));
            Ok(())
        }
        PhysicalRecoveryStagingCommandOutcome::DeniedBeforeEffect(denial) => {
            counters::record_denial(&mut progress.counters, &denial);
            let failure = PhysicalRecoveryStagingDenial::CommandFailed {
                ordinal,
                stage: denial.stage(),
            };
            progress
                .settlements
                .push(PhysicalRecoveryStagingSettlement::DeniedBeforeEffect(
                    denial,
                ));
            Err(failed(
                progress.counters,
                std::mem::take(&mut progress.settlements),
                failure,
            ))
        }
        PhysicalRecoveryStagingCommandOutcome::Indeterminate(indeterminate) => {
            let stage = counters::record_indeterminate(&mut progress.counters, &indeterminate);
            progress
                .settlements
                .push(PhysicalRecoveryStagingSettlement::Indeterminate(
                    indeterminate,
                ));
            Err(failed(
                progress.counters,
                std::mem::take(&mut progress.settlements),
                PhysicalRecoveryStagingDenial::Indeterminate { ordinal, stage },
            ))
        }
    }
}

fn close_staging(
    input: &RecoveryStagingInput,
    mut progress: ExecutionProgress,
) -> Result<StagingExecution, StagingExecution> {
    record_quiescence(
        input,
        &mut progress.counters,
        progress.media_handle_baseline,
    );
    if progress.counters.commands_submitted != progress.counters.planned_scheduler_commands
        || progress.counters.commands_settled != progress.counters.planned_scheduler_commands
        || progress.counters.scheduler_settlements != progress.counters.planned_scheduler_commands
        || progress.counters.live_commands_after_close != 0
        || progress.counters.live_scheduler_reservations_after_close != 0
        || progress.counters.pending_signal_reconciliations_after_close != 0
        || progress.counters.signal_reconciliation_overflow_after_close != 0
        || progress.counters.live_media_handles_after_close != 0
        || !input.coordination.is_ready()
    {
        return Err(failed(
            progress.counters,
            progress.settlements,
            PhysicalRecoveryStagingDenial::QuiescenceMismatch,
        ));
    }
    let closed = ClosedRecoveryStagingGeneration::new(
        input.staging.staging_generation(),
        progress
            .counters
            .artifacts_created
            .saturating_add(progress.counters.artifacts_converged)
            .saturating_add(progress.counters.artifacts_completed_from_prefix),
        progress
            .counters
            .bytes_written
            .saturating_add(progress.counters.bytes_verified),
        progress.content.finalize().into(),
    );
    Ok(StagingExecution {
        counters: progress.counters,
        settlements: PhysicalRecoveryStagingSettlementLedger::new(progress.settlements),
        closed: Some(closed),
        denial: None,
    })
}

fn record_quiescence(
    input: &RecoveryStagingInput,
    counters: &mut PhysicalRecoveryStagingCounters,
    media_handle_baseline: (u64, u64),
) {
    let quiescence = input.coordination.quiescence_observation();
    counters.live_commands_after_close = quiescence.live_commands();
    counters.live_scheduler_reservations_after_close = quiescence.live_scheduler_reservations();
    counters.pending_signal_reconciliations_after_close =
        quiescence.pending_signal_reconciliations();
    counters.signal_reconciliation_overflow_after_close =
        quiescence.signal_reconciliation_overflow();
    let media = input.authority.media.handle_observation();
    counters.live_media_handles_after_close = media
        .live_file_handles()
        .saturating_sub(media_handle_baseline.0)
        .saturating_add(
            media
                .live_directory_handles()
                .saturating_sub(media_handle_baseline.1),
        );
}

fn failed(
    counters: PhysicalRecoveryStagingCounters,
    settlements: Vec<PhysicalRecoveryStagingSettlement>,
    denial: PhysicalRecoveryStagingDenial,
) -> StagingExecution {
    StagingExecution {
        counters,
        settlements: PhysicalRecoveryStagingSettlementLedger::new(settlements),
        closed: None,
        denial: Some(denial),
    }
}
