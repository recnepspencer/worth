//! Current ownership alone fences destruction, including relation endpoint changes.
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::history::data::CanonicalCommitEnvelopeBody;
use crate::history::data::CodecRetirementClosureStop as Stop;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
mod candidate;

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    current: Option<&crate::publication::ownership::read::roster::CurrentPublicationRoster>,
    candidate: &CanonicalCommitEnvelopeBody,
    adjacency: &[worth_relational_contracts::internal_port::authority::mutation::adjacency_delta::AdjacencyDelta],
) -> Result<(), TransactionCommitError> {
    let Some(current) = current else {
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
            current.origins(),
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
                    || crate::transactions::data::outcomes::transaction_commit_error::origin_denial("current ownership selection lost its source"),
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
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
