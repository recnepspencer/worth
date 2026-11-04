//! Current ownership alone fences destruction, including relation endpoint changes.
use crate::{
    history::data::{CanonicalCommitEnvelope, CodecRetirementClosureStop as Stop},
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    publication::patch::data::RecordStructuralChange as Change,
    runtime::RelationalPreparationRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::ExecutionRequest;

pub(super) fn require(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    candidate: &CanonicalCommitEnvelope,
    adjacency: &[crate::authority::mutation::AdjacencyDelta],
) -> Result<(), TransactionCommitError> {
    let Some(current) = runtime.current_ownership.as_deref() else {
        return Ok(());
    };
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        let mut subject = None;
        let selected = crate::history::data::select_owned_scope_dependent(
            crate::history::data::OwnedScopeDependentSelection::LiveRecord(&mut |origin,context| {
                let charge = |context:&mut worth_execution::MapKernelContext<'_, '_>, work:usize|
                    crate::execution::request_stage::checkpoint(context,work as u64)
                        .map_err(|cause| Stop::Request(cause.into()));
                let mut covered = false;
                for successor in candidate.output_origins() {
                    charge(context,origin.meaning.family.len()+successor.meaning.family.len()+65)?;
                    covered |= successor.meaning.same_route(&origin.meaning)
                        && successor.publication.predecessor() == Some(origin.commit.commit_id);
                }
                for retirement in &candidate.required_output_codec_retirements {
                    charge(context,origin.meaning.family.len()+retirement.meaning.family.len()+65)?;
                    covered |= retirement.meaning.same_route(&origin.meaning)
                        && retirement.publication == origin.commit.commit_id;
                }
                if covered { return Ok(false); }
                for patch in &candidate.patch.authoritative_record_patches {
                    charge(context,origin.members.len()+origin.relations.len()+patch.semantic_changes.len()+1)?;
                    let destroys = matches!(patch.structural_change,Change::Deleted|Change::RetainedForAudit)
                        || match patch.target {
                            RecordRef::Relation(id) if patch.structural_change == Change::Updated => {
                                // Native's actual structural patches retain endpoints even when
                                // the schema exposes no endpoint aspect. The final Created delta
                                // is the actual after-image; field-only/no-op updates keep custody.
                                charge(context,adjacency.len()+origin.relations.len()+1)?;
                                let before=origin.relations.iter().find(|edge|edge.id==id);
                                let after=adjacency.iter().rev().find_map(|delta| {
                                    if delta.relation_id!=id {return None;}
                                    match delta.kind {
                                        crate::authority::mutation::AdjacencyDeltaKind::Created{source,target}=>Some((source,target)),
                                        _=>None,
                                    }
                                });
                                before.zip(after).is_some_and(|(before,(source,target))|
                                    before.source!=source || before.target!=target)
                            }
                            _ => false,
                        };
                    if !destroys { continue; }
                    let owned = match patch.target {
                        RecordRef::Entity(id) => id == origin.meaning.source_scope
                            || origin.members.iter().any(|member| member.id == id
                                && member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
                            || origin.relations.iter().any(|edge| edge.source == id || edge.target == id),
                        RecordRef::Relation(id) => origin.relations.iter().any(|edge| edge.id == id),
                    };
                    if owned { subject = Some(patch.target.clone()); return Ok(true); }
                }
                Ok(false)
            }),current,context,
        );
        answer = Some(match selected {
            Ok(Some(origin)) => {
                let target = subject.unwrap_or(RecordRef::Entity(origin.meaning.source_scope));
                let mut error = PublicationError::authored_retirement(Cause::OwnedPublicationRequiresCascade {
                    publication:origin.commit.commit_id,owner:origin.meaning.source_scope,
                },target.clone());
                if target != RecordRef::Entity(origin.meaning.source_scope) {
                    error.context.affected_records.push(RecordRef::Entity(origin.meaning.source_scope));
                }
                Err(TransactionCommitError::publication(error))
            }
            Ok(None) => Ok(()),
            Err(failure) => Err(match failure.stop {
                Stop::Request(cause) => request(cause),
                Stop::Payload(_) => failure.source.map_or_else(
                    || crate::mvcc::origin_denial("current ownership selection lost its source"),
                    |source| TransactionCommitError::publication(PublicationError::authored_retirement(
                        Cause::CorruptPublication,RecordRef::Entity(source)))),
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
