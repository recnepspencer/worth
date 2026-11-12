use super::*;
use crate::orchestration::source_copy::SourceCopyCursor;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

pub(super) fn execute(
    input: &mut RecoveryStagingInput,
    progress: &mut ExecutionProgress,
) -> Result<(), StagingExecution> {
    for index in 0..input.state().staging.source_copies().len() {
        let recipe = input.state().staging.source_copies()[index];
        let source = recipe.intent().source().arena_range();
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena {
                arena: source.arena().get(),
            },
            source.offset(),
            104,
        )
        .ok_or_else(|| invalid(progress))?;
        let observed = read(input, progress, coordinate)?;
        let store = input.state().authority.media.store_identity();
        let format = input.state().authority.record_format;
        let mut cursor = SourceCopyCursor::open(
            store,
            format,
            recipe,
            &observed,
            input.integrity_trace_mut(),
        )
        .map_err(|_| invalid(progress))?;
        drop(observed);
        while let Some(coordinate) = cursor.source_coordinate().map_err(|_| invalid(progress))? {
            let observed = read(input, progress, coordinate)?;
            let (destination, bytes) = cursor
                .transform(&observed, input.integrity_trace_mut())
                .map_err(|_| invalid(progress))?;
            drop(observed);
            write(input, progress, destination, &bytes)?;
        }
        let (destination, bytes) = cursor.finish().map_err(|_| invalid(progress))?;
        write(input, progress, destination, &bytes)?;
    }
    Ok(())
}

fn read(
    input: &RecoveryStagingInput,
    progress: &mut ExecutionProgress,
    coordinate: RecordFrameCoordinate,
) -> Result<worth_store::physical_runtime::ObservedRecoveryArtifact, StagingExecution> {
    progress.counters.commands_submitted += 1;
    let completed = input
        .state()
        .coordination
        .owner()
        .execute_source_copy_read(
            &input.state().authority.media,
            coordinate,
            u64::from(input.state().authority.record_format.page_size().bytes()),
        )
        .map_err(|_| invalid(progress))?;
    progress.counters.commands_settled += 1;
    progress.counters.scheduler_settlements += 1;
    Ok(completed.observed())
}

fn write(
    input: &RecoveryStagingInput,
    progress: &mut ExecutionProgress,
    coordinate: RecordFrameCoordinate,
    bytes: &[u8],
) -> Result<(), StagingExecution> {
    let ordinal = progress.settlements.len() as u64;
    let declaration = PhysicalRecoveryStagingCommand::new(
        ordinal,
        input.state().publication.plan_identity(),
        input.state().staging.staging_generation(),
        coordinate.artifact(),
        coordinate.offset(),
        bytes,
        Sha256::digest(bytes).into(),
    )
    .ok_or_else(|| invalid(progress))?;
    let outcome = input
        .state()
        .coordination
        .owner()
        .execute_staging_command(&input.state().authority.media, declaration);
    record_command_outcome(progress, ordinal, outcome)?;
    if matches!(input.state().cancellation,super::super::RecoveryStagingCancellation::AfterSettledCommands(settled)
        if settled==progress.settlements.len() as u64)
    {
        return Err(failed(
            progress.counters,
            std::mem::take(&mut progress.settlements),
            PhysicalRecoveryStagingDenial::CancelledAfterPartialStaging {
                settled_commands: progress.counters.commands_settled,
            },
        ));
    }
    Ok(())
}

fn invalid(progress: &mut ExecutionProgress) -> StagingExecution {
    failed(
        progress.counters,
        std::mem::take(&mut progress.settlements),
        PhysicalRecoveryStagingDenial::InvalidPlan,
    )
}
