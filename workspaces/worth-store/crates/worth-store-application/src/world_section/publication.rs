//! One invoking call advances the actual task retained by the attached Home.
use super::publication_task::{Endpoint, WorldPublicationTask};
use super::ApplicationWorldMedium;
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, ExecutionResourceLease};
use worth_runtime_world::facade::WorldPublicationCompletion;
use worth_runtime_world::facade::{
    WorldDurableArtifact, WorldDurableMediumDenial as Denial, WorldPublicationObservation,
};
use worth_store::physical_runtime::*;

pub(super) fn publish(
    medium: &ApplicationWorldMedium,
    subject: &WorldDurableArtifact,
    execution: &ExecutionRequest<'_, '_>,
) -> WorldPublicationObservation {
    let task = match medium.publications.task(subject) {
        Ok(task) => task,
        Err(_) => return WorldPublicationObservation::Pending,
    };
    let Some(_driver) = task.try_driver() else {
        return WorldPublicationObservation::Pending;
    };
    // Recovery is consumed outside the disposable return context. Its SAME
    // returned strong retainer is installed before the operation is lent.
    task.resume();
    if let Some(terminal) = task.observation() {
        return terminal;
    }
    let serving = match medium.serving() {
        Ok(serving) => serving,
        Err(cause) => return no_effect(cause),
    };
    let mut returned = None;
    let mut run = |lease: Option<&ExecutionResourceLease<'_>>| {
        execution
            .run_native_with_checkpoint(|context| {
                returned = Some((|| {
                    let journal = serving
                        .ordered_journal(
                            OrderedJournalName::new(subject.journal().name())
                                .expect("closed World journal"),
                        )
                        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
                    if !task.started() {
                        let (position, publication) =
                            prepare(medium, &journal, subject, execution)?;
                        let view = context
                            .retain_interruption_view(lease)
                            .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
                        match journal.start_publication(publication, view) {
                            OrderedJournalPublicationStart::Ready {
                                operation,
                                recovery,
                            } => {
                                let valid = matches_source(
                                    &serving,
                                    subject,
                                    position,
                                    operation.observe().source(),
                                );
                                // Registration precedes the first physical step, including
                                // a source mismatch's irreversible stop/settlement path.
                                task.install(operation, recovery);
                                if !valid {
                                    task.invalidate();
                                    task.stop();
                                    return Err(Denial::PublicationUnresolved);
                                }
                            }
                            OrderedJournalPublicationStart::AlreadyCommitted(committed) => {
                                let valid =
                                    matches_source(&serving, subject, position, committed.source());
                                task.committed(committed);
                                if !valid {
                                    task.invalidate();
                                }
                                return Ok(None);
                            }
                            OrderedJournalPublicationStart::Refused(cause) => {
                                return Ok(Some(super::publication_fate::start_failure(&cause)));
                            }
                        }
                    }
                    advance(&task, &journal);
                    Ok(None)
                })());
            })
            .map(|_| ())
    };
    let scope = if execution.is_leased() {
        execution.in_scope(run).and_then(|result| result)
    } else {
        run(None)
    };
    // A completed owner task outranks a late stop of the invoking request.
    if let Some(terminal) = task.observation() {
        return terminal;
    }
    if scope.is_err() {
        // No later caller may resume advancement after the invoking context
        // was lost. Already accepted work is settled by the same task.
        task.stop();
    }
    if let Some(terminal) = task.observation() {
        return terminal;
    }
    match (scope, returned) {
        (_, Some(Ok(Some(observation)))) => observation,
        (Err(cause), _) if !task.started() => no_effect(Denial::Request(cause)),
        (_, Some(Err(detail))) if !task.started() => no_effect(detail),
        (_, Some(Err(_))) => {
            task.stop();
            WorldPublicationObservation::Pending
        }
        _ => WorldPublicationObservation::Pending,
    }
}

pub(super) fn advance(task: &WorldPublicationTask, journal: &OrderedJournal<'_>) {
    let Some(endpoint) = task.take_endpoint() else {
        return;
    };
    match endpoint {
        Endpoint::Ready(operation) => match operation.step(journal) {
            OrderedJournalPublicationStep::Pending(operation) => {
                task.restore(Endpoint::Ready(operation))
            }
            OrderedJournalPublicationStep::Completed(committed) => task.committed(committed),
            OrderedJournalPublicationStep::Failed(failure) => {
                task.restore(Endpoint::Stopped(failure.into_stopped()));
            }
        },
        Endpoint::Stopped(stopped) => match stopped.settle(journal) {
            OrderedJournalStoppedPoll::Pending(stopped) => task.restore(Endpoint::Stopped(stopped)),
            OrderedJournalStoppedPoll::Settled(observation) => task.settled(observation),
            OrderedJournalStoppedPoll::Failed(failure) => {
                let observation = failure.observe();
                if observation.terminal().is_some() {
                    task.settled(observation);
                } else {
                    task.restore(Endpoint::Stopped(failure.into_stopped()));
                }
            }
        },
        terminal => task.restore(terminal),
    }
}

fn prepare(
    medium: &ApplicationWorldMedium,
    journal: &OrderedJournal<'_>,
    subject: &WorldDurableArtifact,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(u64, OrderedJournalPublication), Denial> {
    execution.consult().map_err(Denial::Request)?;
    let position = journal
        .head()
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?
        .map_or(Some(1), |head| head.position().checked_add(1))
        .ok_or(Denial::Unavailable)?;
    let input = crate::frame_encoding::encode(subject.as_bytes().len(), execution, |out| {
        out.extend_from_slice(subject.as_bytes())
    })
    .map_err(Denial::Request)?;
    // Signal's policy is independent of the retained original absolute view.
    let deadline = PhysicalMutationDeadline::after_milliseconds(120_000)
        .expect("fixed positive physical policy");
    let mut publication =
        OrderedJournalPublication::new(position, input.bytes, medium.placement, deadline);
    if let Some(memory) = input.memory {
        publication = publication.with_input_memory(memory);
    }
    Ok((position, publication))
}
fn matches_source(
    serving: &ServingPhysicalRuntime,
    subject: &WorldDurableArtifact,
    position: u64,
    source: OrderedJournalPublicationSource,
) -> bool {
    let digest: [u8; 32] = Sha256::digest(subject.as_bytes()).into();
    source.runtime_identity() == serving.runtime_identity()
        && source.journal().as_str() == subject.journal().name()
        && source.position() == position
        && source.payload_bytes() == subject.as_bytes().len() as u64
        && source.payload_sha256() == digest
}
fn no_effect(detail: Denial) -> WorldPublicationObservation {
    WorldPublicationObservation::Finished(WorldPublicationCompletion::proven_no_effect(detail))
}
