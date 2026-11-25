//! Current ownership alone fences destruction, including relation endpoint changes.
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody;
use branch::history::data::codec_retirement_closure::CodecRetirementClosureStop as Stop;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::publication::patch::data::record_structural_change::RecordStructuralChange as Change;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
pub mod candidate;

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    current: Option<&branch::publication::ownership::read::roster::CurrentPublicationRoster>,
    candidate: &CanonicalCommitEnvelopeBody,
    adjacency: &[contracts::authority::mutation::adjacency_delta::AdjacencyDelta],
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
        let selected = branch::history::data::codec_retirement_closure::scope_dependents::select_owned_scope_dependent(
            branch::history::data::codec_retirement_closure::scope_dependents::OwnedScopeDependentSelection::LiveRecord(
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
                    || contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("current ownership selection lost its source"),
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
    cause: impl Into<contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
