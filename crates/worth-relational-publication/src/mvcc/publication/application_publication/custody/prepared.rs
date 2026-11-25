//! Fund canonical origin traversal and Native custody before the World transition.
use super::*;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational_branch::internal_port::history::data::SharedCanonicalCommitEnvelope;
use worth_relational_contracts::internal_port as contracts;
struct PreparedOrigin {
    ordinal: usize,
    origin: Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin>,
}
pub struct RelationalPreparedOutputPublications {
    installed: RelationalInstalledApplicationPublicationAuthority,
    envelope: SharedCanonicalCommitEnvelope,
    origin_count: usize,
    selected: ExecutionArray<PreparedOrigin>,
}
pub struct RelationalPerformedOutputPublications {
    prepared: RelationalPreparedOutputPublications,
    envelope: SharedCanonicalCommitEnvelope,
}
pub fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    envelope: &Arc<CanonicalCommitEnvelope>,
) -> Result<RelationalPreparedOutputPublications, TransactionCommitError> {
    let (origin_count, selected_count) = execution
        .run_native_with_checkpoint(|context| {
            let mut total = 0usize;
            let mut selected = 0usize;
            for origin in envelope.output_origins() {
                contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(execution_stop)?;
                total = total.checked_add(1).ok_or_else(overflow)?;
                if origin
                    .issuer
                    .as_ref()
                    .is_some_and(|issuer| issuer.same_publication_owner(&installed.authority))
                {
                    selected = selected.checked_add(1).ok_or_else(overflow)?;
                }
            }
            // Completion checks immutable positions and seals only these selected
            // origins after World. All of that fixed work belongs to this request now.
            contracts::execution::request_stage::checkpoint(
                context,
                u64::try_from(
                    total
                        .checked_mul(3)
                        .and_then(|n| n.checked_add(selected))
                        .ok_or_else(overflow)?,
                )
                .map_err(|_| overflow())?,
            )
            .map_err(execution_stop)?;
            Ok::<_, TransactionCommitError>((total, selected))
        })
        .map_err(execution_stop)?
        .0?;
    let mut selected =
        ExecutionRequestArrayBuilder::allocate(*execution, selected_count).map_err(array_stop)?;
    execution
        .run_native_with_checkpoint(|context| {
            for (ordinal, origin) in envelope.output_origins().enumerate() {
                contracts::execution::request_stage::checkpoint(context, 1)
                    .map_err(execution_stop)?;
                if origin
                    .issuer
                    .as_ref()
                    .is_some_and(|issuer| issuer.same_publication_owner(&installed.authority))
                {
                    selected
                        .push(PreparedOrigin {
                            ordinal,
                            origin: Arc::clone(origin),
                        })
                        .map_err(array_stop)?;
                }
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(execution_stop)?
        .0?;
    Ok(RelationalPreparedOutputPublications {
        installed: installed.clone(),
        envelope: SharedCanonicalCommitEnvelope::from_arc(Arc::clone(envelope)),
        origin_count,
        selected: selected.seal().map_err(array_stop)?,
    })
}
impl RelationalPreparedOutputPublications {
    /// Descriptive primary roles before World, from this exact prepared envelope.
    /// This borrows no publication authority and cannot finish the preparation.
    pub fn primary_output_members(
        &self,
    ) -> Option<
        impl ExactSizeIterator<
            Item = (
                &str,
                &str,
                contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture,
                contracts::identity::data::record_ids::EntityId,
            ),
        >,
    >{
        (*self.envelope.owner_required_output_origin())
            .as_ref()
            .map(|origin| {
                origin
                    .members
                    .iter()
                    .map(|m| (m.role.as_str(), m.entity_name.as_str(), m.posture, m.id))
            })
    }

    /// Descriptive canonical roles of one prepared participant, without custody.
    pub fn output_members_for(
        &self,
        producer: &str,
        meaning: &contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    ) -> Option<
        impl ExactSizeIterator<
            Item = (
                &str,
                &str,
                contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture,
                contracts::identity::data::record_ids::EntityId,
            ),
        >,
    >{
        self.selected
            .elements()
            .iter()
            .find(|p| p.origin.producer == producer && p.origin.meaning.same_route(meaning))
            .map(|p| {
                p.origin
                    .members
                    .iter()
                    .map(|m| (m.role.as_str(), m.entity_name.as_str(), m.posture, m.id))
            })
    }

    /// Only the actual performed result can complete this preparation. No new
    /// request work or allocation can refuse after that result's World effect.
    pub fn finish(self, performed: &CommitResult) -> Option<RelationalPerformedOutputPublications> {
        let envelope = &performed.publication().envelope;
        if performed.snapshot.runtime_instance_id() != self.installed.runtime_instance_id
            || envelope.commit != self.envelope.commit
            || envelope.output_origins().count() != self.origin_count
            || !envelope
                .output_origins()
                .zip(self.envelope.output_origins())
                .all(|(actual, expected)| Arc::ptr_eq(actual, expected))
        {
            return None;
        }
        Some(RelationalPerformedOutputPublications {
            prepared: self,
            envelope: envelope.clone(),
        })
    }
}
impl RelationalPerformedOutputPublications {
    pub fn into_publications(
        self,
    ) -> impl ExactSizeIterator<Item = RelationalRequiredOutputPublication> {
        let RelationalPreparedOutputPublications {
            installed,
            selected,
            ..
        } = self.prepared;
        selected
            .into_iter()
            .map(move |selected| RelationalRequiredOutputPublication {
                runtime_instance_id: installed.runtime_instance_id,
                evidence: worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Canonical(
                    self.envelope.clone(),
                ),
                authority: installed.authority.seal_required_output_publication_at(
                    self.envelope.commit.commit_id.0,
                    selected.ordinal,
                ),
                origin: selected.origin,
                readmitted_at: None,
            })
    }
}
fn overflow() -> TransactionCommitError {
    execution_stop(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}

fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> TransactionCommitError {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => execution_stop(cause),
        worth_execution::ExecutionRequestArrayDenial::Allocation(denial) => {
            TransactionCommitError::conflict(contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                contracts::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied { denial },
            ))
        }
    }
}
