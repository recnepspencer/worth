//! Count-first scheduling reads only bounded descriptive suffix headers.
use super::{ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits as Limits};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldDurableJournal;
use worth_store::physical_runtime::*;

#[derive(Clone, Copy, Debug)]
pub struct ApplicationHomeCheckpointJournalObservation {
    pub name: &'static str,
    pub covered: u64,
    pub head: u64,
    /// None when a trigger avoided completing this journal's byte inspection.
    pub uncovered_payload_bytes: Option<u64>,
}
pub(crate) struct CheckpointCadence {
    pub(crate) journals: [ApplicationHomeCheckpointJournalObservation; 5],
    pub(crate) due: bool,
}
pub(crate) fn inspect(
    serving: &ServingPhysicalRuntime,
    covered: [u64; 5],
    limits: Limits,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CheckpointCadence, Denial> {
    request.consult().map_err(Denial::Execution)?;
    let reader = serving
        .try_records_for_checkpoint(request)
        .map_err(|cause| match cause {
            PhysicalCheckpointReadProtectionDenial::Request(stop) => {
                Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
            }
            cause => Denial::CheckpointReadProtection(cause),
        })?;
    let names = [
        RELATIONAL_CANONICAL_COMMITS,
        RELATIONAL_BRANCH_TRANSITIONS,
        WorldDurableJournal::Branches.name(),
        WorldDurableJournal::Custody.name(),
        WorldDurableJournal::Occurrences.name(),
    ];
    let mut observations = names.map(|name| ApplicationHomeCheckpointJournalObservation {
        name,
        covered: 0,
        head: 0,
        uncovered_payload_bytes: None,
    });
    // Live idempotency bindings are reclaimed only after `retention` subsequent
    // checkpoints commit. Space checkpoints across the remaining binding budget
    // by counting only bindings issued in the current generation, so a finished
    // checkpoint immediately resets this trigger instead of thrashing back-to-back.
    let idempotency = serving.durability_observation().idempotency_policy();
    let live_limit = u64::from(idempotency.live_binding_limit().get().get());
    let retention = idempotency.retention().get().get();
    let (live_bindings, current_gen_bindings) = serving.idempotency_binding_counts();
    let remaining_bindings = live_limit.saturating_sub(live_bindings);
    let generation_budget = (remaining_bindings / retention.saturating_add(1)).max(1);
    let mut count_due =
        current_gen_bindings >= generation_budget || serving.physical_retention_checkpoint_due();
    // Check ALL five counts before requesting any suffix header. One count
    // trigger therefore avoids byte inspection even in another journal.
    for (index, observation) in observations.iter_mut().enumerate() {
        request.consult().map_err(Denial::Execution)?;
        observation.covered = covered[index];
        let journal = serving
            .ordered_journal(name(observation.name))
            .map_err(Denial::JournalUnavailable)?;
        observation.head = journal
            .head_position_from(&reader)
            .map_err(Denial::JournalRead)?;
        let delta = observation
            .head
            .checked_sub(observation.covered)
            .ok_or(Denial::CoverageAheadOfJournal)?;
        count_due |= delta >= limits.entries().get();
    }
    if count_due {
        return Ok(CheckpointCadence {
            journals: observations,
            due: true,
        });
    }
    // Every uncovered payload byte waits in the same physical WAL tail, so
    // the byte trigger compares the total across all five journals.
    let mut total = 0u64;
    for observation in &mut observations {
        let journal = serving
            .ordered_journal(name(observation.name))
            .map_err(Denial::JournalUnavailable)?;
        let mut position = observation.covered;
        let mut bytes = 0u64;
        while position < observation.head {
            position = position.checked_add(1).ok_or(Denial::SizeLimit)?;
            request
                .run_native_with_checkpoint(|context| context.checkpoint(1))
                .map_err(Denial::Execution)?
                .0
                .map_err(|stop| {
                    Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(stop))
                })?;
            let entry = journal
                .describe_entry_from(&reader, request, position)
                .map_err(Denial::JournalRead)?;
            if entry.position() != position {
                return Err(Denial::JournalDescriptionMismatch);
            }
            bytes = bytes
                .checked_add(entry.payload_bytes())
                .ok_or(Denial::SizeLimit)?;
            total = total
                .checked_add(entry.payload_bytes())
                .ok_or(Denial::SizeLimit)?;
            if total >= limits.payload_bytes().get() {
                break;
            }
        }
        if position == observation.head {
            observation.uncovered_payload_bytes = Some(bytes);
        }
        if total >= limits.payload_bytes().get() {
            break;
        }
    }
    Ok(CheckpointCadence {
        journals: observations,
        due: total >= limits.payload_bytes().get(),
    })
}
fn name(name: &'static str) -> OrderedJournalName {
    OrderedJournalName::new(name).expect("closed checkpoint journal")
}
