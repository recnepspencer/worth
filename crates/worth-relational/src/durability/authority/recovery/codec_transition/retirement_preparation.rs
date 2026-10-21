//! Capture only complete decisions from the native retirement court.
use super::publication_custody::RecoveredCodecPublicationCustody;
use crate::history::data::{
    CanonicalCommitEnvelope, RequiredOutputPublicationPositions, RequiredOutputRoutePosition,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::TransactionCommitError;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{
        CanonicalCodecRetirementKind, CanonicalRequiredOutputCodecRetirement,
        CanonicalRequiredOutputOrigin,
    },
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(super) struct PreparedCodecRetirement {
    origin: Arc<CanonicalRequiredOutputOrigin>,
    kinds: Arc<Vec<CanonicalCodecRetirementKind>>,
}

pub(super) fn capture(
    execution: &ExecutionRequest<'_, '_>,
    source: &AdmittedRelationalBranchBasis,
    target: &RelationalSchemaRegistry,
    retirement: &crate::history::data::OwnedCodecRetirementClosure,
    activation: &crate::runtime::RecoveredCheckpointProgramActivationUpdate,
) -> Result<
    (
        Vec<PreparedCodecRetirement>,
        Arc<ExecutionMemoryReservation>,
    ),
    crate::runtime::RelationalSchemaTransitionAdmissionDenial,
> {
    use crate::runtime::RelationalSchemaTransitionAdmissionDenial as Denial;
    let result = execution.in_scope(|lease| {
        worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(lease, None, |context| {
            let (origins, _roster) = retirement.complete_publications(context, lease).map_err(|stop| match stop {
                crate::history::data::CodecRetirementClosureStop::Request(cause) => Denial::request(cause),
                crate::history::data::CodecRetirementClosureStop::Payload(detail) => Denial::new(crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected, detail),
            })?;
            let activation_kind = source.inner.root.partition_state(activation.entity_id.partition_id)
                .and_then(|partition| partition.entity_arena.get(&activation.entity_id))
                .filter(|row| row.is_live()).and_then(|row| row.kind_id())
                .ok_or_else(|| Denial::new(crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected, "codec activation actual native kind is absent"))?;
            let overflow = || Denial::request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow);
            let activation_bytes = std::mem::size_of_val(activation).checked_add(2 * std::mem::size_of::<usize>()).and_then(|n| n.checked_add(activation.field_locator.owned_allocation_capacity_bytes())).and_then(|n| n.checked_add(activation.expected.semantic_byte_width())).and_then(|n| n.checked_add(activation.replacement.semantic_byte_width())).ok_or_else(overflow)?;
            let mut bytes = origins.len().checked_mul(std::mem::size_of::<PreparedCodecRetirement>() + std::mem::size_of::<CanonicalRequiredOutputCodecRetirement>() + 512).and_then(|n| n.checked_add(activation_bytes)).ok_or_else(overflow)?;
            for origin in &origins {
                bytes = bytes.checked_add(origin.meaning.family.len()).and_then(|n| n.checked_add(source.descriptor().branch_id().0.len().saturating_mul(4))).ok_or_else(overflow)?;
                for (index, kind_id) in origin.members.iter().map(|member| member.kind).chain(std::iter::once(activation_kind)).enumerate() {
                    context.checkpoint((index + 1) as u64).map_err(|stop| Denial::request(stop.into()))?;
                    if origin.members[..index.min(origin.members.len())].iter().any(|old| old.kind == kind_id) { continue; }
                    for registry in [source.inner.root.schema_authority().registry(), target] {
                        let kind = registry.entity_kinds.get(&kind_id).ok_or_else(|| Denial::new(crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected, "codec group has no actual kind contract"))?;
                        let quote = crate::durability::data::DurableEntitySchemaRegistration::capture_bytes(kind).ok_or_else(overflow)?;
                        context.checkpoint(quote as u64).map_err(|stop| Denial::request(stop.into()))?;
                        bytes = bytes.checked_add(quote).ok_or_else(overflow)?;
                    }
                }
            }
            let funding = Arc::new(ExecutionMemoryReservation::reserve_in_scope(lease, bytes as u64).map_err(|stop| Denial::request(crate::execution::denial::lease_denial(stop)))?);
            let mut prepared = Vec::with_capacity(origins.len());
            for origin in origins {
                let mut kinds = Vec::new();
                for (index, kind_id) in origin.members.iter().map(|member| member.kind).chain(std::iter::once(activation_kind)).enumerate() {
                    context.checkpoint((index + 1) as u64).map_err(|stop| Denial::request(stop.into()))?;
                    if origin.members[..index.min(origin.members.len())].iter().any(|old| old.kind == kind_id) { continue; }
                    kinds.push(CanonicalCodecRetirementKind {
                        source: crate::durability::data::DurableEntitySchemaRegistration::capture(&source.inner.root.schema_authority().registry().entity_kinds[&kind_id]),
                        target: crate::durability::data::DurableEntitySchemaRegistration::capture(&target.entity_kinds[&kind_id]),
                    });
                }
                prepared.push(PreparedCodecRetirement { origin, kinds: Arc::new(kinds) });
            }
            Ok((prepared, funding))
        })
    });
    execution
        .consult()
        .map_err(|stop| Denial::request(stop.into()))?;
    result
        .map_err(|stop| Denial::request(stop.into()))?
        .map_err(|stop| Denial::request(stop.into()))?
        .0
}

impl RecoveredCodecPublicationCustody {
    pub(crate) fn install_retirements(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        envelope: &mut CanonicalCommitEnvelope,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
        use crate::publication::patch::data::RecordStructuralChange as Change;
        use crate::transactions::data::RecordRef;
        let denied = |detail| crate::mvcc::origin_denial(detail);
        if self.source.descriptor() != source.descriptor()
            || envelope.schema_transition.is_none()
            || !envelope.is_unmerged_output_publication_step()
            || envelope.required_output_origin.is_some()
            || envelope
                .branch_cell_checkpoint
                .as_ref()
                .map(|cell| &cell.observation)
                != Some(source.reference())
        {
            return Err(denied(
                "codec retirement custody does not cover this exact native candidate",
            ));
        }
        let result = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    let mut positions = source.inner.root.output_publication_positions.clone();
                    for prepared in &self.retirements {
                        let origin = &prepared.origin;
                        for member in &origin.members {
                            context
                                .checkpoint(
                                    envelope.patch.authoritative_record_patches.len() as u64 + 1,
                                )
                                .map_err(retirement_request)?;
                            if member.posture == crate::mvcc::RequiredOutputRolePosture::Retire {
                                continue;
                            }
                            if envelope
                                .patch
                                .authoritative_record_patches
                                .iter()
                                .filter(|patch| {
                                    patch.target == RecordRef::Entity(member.id)
                                        && patch.structural_change == Change::Deleted
                                })
                                .count()
                                != 1
                            {
                                return Err(denied(
                                    "codec retirement omits an actual complete member deletion",
                                ));
                            }
                        }
                        for relation in &origin.relations {
                            if envelope
                                .patch
                                .authoritative_record_patches
                                .iter()
                                .filter(|patch| {
                                    patch.target == RecordRef::Relation(relation.id)
                                        && matches!(
                                            patch.structural_change,
                                            Change::Deleted | Change::RetainedForAudit
                                        )
                                })
                                .count()
                                != 1
                            {
                                return Err(denied(
                                    "codec retirement omits an actual complete relation deletion",
                                ));
                            }
                        }
                        positions.insert(
                            &origin.meaning,
                            RequiredOutputRoutePosition::CodecRetired(envelope.commit.commit_id),
                            lease,
                            context,
                        )?;
                        envelope.required_output_codec_retirements.push(Arc::new(
                            CanonicalRequiredOutputCodecRetirement {
                                publication: origin.commit.commit_id,
                                meaning: origin.meaning.clone(),
                                source: source.descriptor().clone(),
                                retiring: envelope.commit.clone(),
                                kinds: Arc::clone(&prepared.kinds),
                                activation: Arc::clone(&self.activation),
                                _funding: Some(Arc::clone(&self.retirement_funding)),
                            },
                        ));
                    }
                    Ok(positions)
                })
        });
        execution.consult().map_err(retirement_request)?;
        result
            .map_err(retirement_request)?
            .map_err(retirement_request)?
            .0
    }
}

fn retirement_request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
