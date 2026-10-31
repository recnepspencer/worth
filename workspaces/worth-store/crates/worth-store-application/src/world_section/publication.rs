use super::ApplicationWorldMedium;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurablePublicationFate;
use worth_runtime_world::facade::{
    WorldDurableArtifact, WorldDurableMediumDenial as Denial, WorldPublicationObservation,
};
use worth_store::physical_runtime::{
    OrderedJournalName, OrderedJournalPublication, OrderedJournalPublicationFailure as Failure,
    OrderedJournalPublished, PhysicalMutationDeadline,
};
pub(super) fn publish(
    medium: &ApplicationWorldMedium,
    subject: &WorldDurableArtifact,
    execution: &ExecutionRequest<'_, '_>,
) -> WorldPublicationObservation {
    WorldPublicationObservation::Finished(publish_actual(medium, subject, execution))
}
fn publish_actual(
    medium: &ApplicationWorldMedium,
    subject: &WorldDurableArtifact,
    execution: &ExecutionRequest<'_, '_>,
) -> DurablePublicationFate<Denial> {
    let prepared = prepare(medium, subject, execution);
    let (journal, publication, _input_memory) = match prepared {
        Ok(prepared) => prepared,
        Err(detail) => return DurablePublicationFate::ProvenNoEffect { detail },
    };
    // Only this actual journal call supplies the fate. Its returned entry is
    // tied to this exact offered position and payload by the physical owner.
    match journal.publish(publication) {
        Ok(
            OrderedJournalPublished::Committed(_) | OrderedJournalPublished::AlreadyCommitted(_),
        ) => DurablePublicationFate::Committed,
        Err(failure) => failure_fate(failure),
    }
}
fn prepare<'a>(
    medium: &'a ApplicationWorldMedium,
    subject: &WorldDurableArtifact,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        worth_store::physical_runtime::OrderedJournal<'a>,
        OrderedJournalPublication,
        Option<worth_execution::ExecutionMemoryReservation>,
    ),
    Denial,
> {
    execution.consult().map_err(Denial::Request)?;
    let journal = medium
        .serving
        .ordered_journal(
            OrderedJournalName::new(subject.journal().name()).expect("closed World journal"),
        )
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
    let position = journal
        .head()
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?
        .map_or(Some(1), |head| head.position().checked_add(1))
        .ok_or(Denial::Unavailable)?;
    let input = crate::frame_encoding::encode(subject.as_bytes().len(), execution, |out| {
        out.extend_from_slice(subject.as_bytes())
    })
    .map_err(Denial::Request)?;
    let milliseconds = crate::deadline::milliseconds(execution, 30_000)
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
    let deadline =
        PhysicalMutationDeadline::after_milliseconds(milliseconds).ok_or(Denial::Unavailable)?;
    Ok((
        journal,
        OrderedJournalPublication::new(position, input.bytes, medium.placement, deadline),
        input.memory,
    ))
}
fn failure_fate(failure: Failure) -> DurablePublicationFate<Denial> {
    let detail = Denial::Storage(format!("{failure:?}"));
    match failure {
        Failure::Indeterminate(_)
        | Failure::CompletedWithoutRecord
        | Failure::Unresolved { .. } => DurablePublicationFate::Indeterminate { detail },
        Failure::EmptyPayload
        | Failure::PositionGap { .. }
        | Failure::PositionConflict(_)
        | Failure::AnchorTableFull
        | Failure::Batch(_)
        | Failure::Idempotency(_)
        | Failure::Preparation(_)
        | Failure::ProvenNoEffect(_)
        | Failure::Read(_) => DurablePublicationFate::ProvenNoEffect { detail },
        // OrderedJournalPublicationFailure is non-exhaustive. Unknown future
        // failures cannot prove that the original publication had no effect.
        _ => DurablePublicationFate::Indeterminate { detail },
    }
}
