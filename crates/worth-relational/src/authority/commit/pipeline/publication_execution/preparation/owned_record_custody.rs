//! Current ownership alone fences destruction, including relation endpoint changes.
use crate::{
    history::data::{CanonicalCommitEnvelope, CodecRetirementClosureStop as Stop},
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    publication::patch::data::RecordStructuralChange as Change,
    runtime::RelationalPreparationRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::ExecutionRequest;
mod candidate;

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    candidate: &CanonicalCommitEnvelope,
    adjacency: &[crate::authority::mutation::AdjacencyDelta],
) -> Result<(), TransactionCommitError> {
    let Some(current) = runtime.current_ownership.as_deref() else {
        return Ok(());
    };
    if current.origins().len() == 0 {
        return Ok(());
    }
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        let writes = match candidate::CandidateWrites::new(execution, candidate, adjacency, context)
        {
            Ok(writes) => writes,
            Err(error) => {
                answer = Some(Err(error));
                return;
            }
        };
        let mut subject = None;
        let selected = crate::history::data::select_owned_scope_dependent(
            crate::history::data::OwnedScopeDependentSelection::LiveRecord(
                &mut |origin, context| {
                    if writes.covered(origin) {
                        return Ok(false);
                    }
                    subject = writes.destruction(origin, context)?;
                    Ok(subject.is_some())
                },
            ),
            current,
            context,
        );
        answer = Some(match selected {
            Ok(Some(origin)) => {
                let target = subject.unwrap_or(RecordRef::Entity(origin.meaning.source_scope));
                let mut error = PublicationError::authored_retirement(
                    Cause::OwnedPublicationRequiresCascade {
                        publication: origin.commit.commit_id,
                        owner: origin.meaning.source_scope,
                    },
                    target.clone(),
                );
                if target != RecordRef::Entity(origin.meaning.source_scope) {
                    error
                        .context
                        .affected_records
                        .push(RecordRef::Entity(origin.meaning.source_scope));
                }
                Err(TransactionCommitError::publication(error))
            }
            Ok(None) => Ok(()),
            Err(failure) => Err(match failure.stop {
                Stop::Request(cause) => request(cause),
                Stop::Payload(_) => failure.source.map_or_else(
                    || crate::mvcc::origin_denial("current ownership selection lost its source"),
                    |source| {
                        TransactionCommitError::publication(PublicationError::authored_retirement(
                            Cause::CorruptPublication,
                            RecordRef::Entity(source),
                        ))
                    },
                ),
            }),
        });
    });
    match (handoff, answer) {
        (Ok(_), Some(answer)) => answer,
        (Err(_), Some(Err(cause))) => Err(cause),
        (Err(cause), _) => Err(request(cause)),
        (Ok(_), None) => unreachable!("completed fence evaluated current facts"),
    }
}
fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
