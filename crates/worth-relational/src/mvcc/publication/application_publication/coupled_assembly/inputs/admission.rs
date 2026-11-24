//! Admit consumed-output route custody against the real applied part sequence.
use super::super::super::{
    custody, origin_denial, RelationalRequiredOutputPublication, RequiredOutputPublicationMeaning,
    RequiredOutputRolePosture,
};
use super::super::inputs::{
    InputIssuance, InputOrigin, RelationalRequiredOutputInput, RequiredOutputInputs,
};
use super::super::refusals::reject;
use super::super::retained_assembly::{BoundAuthoredCoupling, CoupledPublicationPart};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(super) fn restored_index(
    binding: &BoundAuthoredCoupling,
    execution: &ExecutionRequest<'_, '_>,
    family: &str,
    original: EntityId,
    source: EntityId,
    partition: &[u8; 32],
) -> Result<usize, TransactionCommitError> {
    let restoration = binding.owners.source.restoration().expect("restored owner");
    execution
        .run_native_with_checkpoint(|context| {
            for (index, publication) in restoration.publications.iter().enumerate() {
                let meaning = &publication.origin.meaning;
                checkpoint(context, (family.len() + meaning.family.len() + 65) as u64)
                    .map_err(custody::execution_stop)?;
                if meaning.family == family
                    && meaning.source_scope == original
                    && meaning.source_partition.bytes() == partition
                {
                    return Ok(index);
                }
            }
            Err(reject(Cause::ForeignCoupledOwner, source))
        })
        .map_err(custody::execution_stop)?
        .0
}

pub(super) fn require_current(
    binding: &BoundAuthoredCoupling,
    execution: &ExecutionRequest<'_, '_>,
    publication: &RelationalRequiredOutputPublication,
) -> Result<(), TransactionCommitError> {
    let origin = &publication.origin;
    let subject = origin.meaning.source_scope;
    let current = execution
        .run_native_with_checkpoint(|context| {
            checkpoint(context, origin.members.len() as u64 + 1)
                .map_err(custody::execution_stop)?;
            if origin
                .members
                .iter()
                .all(|member| member.posture == RequiredOutputRolePosture::Retire)
            {
                return Ok(false);
            }
            if publication.readmitted_at.as_ref() == Some(&binding.owners.before) {
                return Ok::<_, TransactionCommitError>(true);
            }
            for selected in &binding.owners.current {
                checkpoint(
                    context,
                    (origin.meaning.family.len() + selected.origin.meaning.family.len() + 65)
                        as u64,
                )
                .map_err(custody::execution_stop)?;
                if selected.origin.same_publication(origin) {
                    return Ok(true);
                }
            }
            Ok(false)
        })
        .map_err(custody::execution_stop)?
        .0?;
    if !current
        || publication.runtime_instance_id != binding.installed.runtime_instance_id
        || !binding
            .installed
            .authority
            .owns_required_output_publication(publication.authority())
    {
        return Err(reject(Cause::ForeignCoupledOwner, subject));
    }
    Ok(())
}

fn input_route<'a>(
    parts: &'a [CoupledPublicationPart],
    binding: &'a BoundAuthoredCoupling,
    input: &'a InputOrigin,
) -> (
    &'a RequiredOutputPublicationMeaning,
    crate::identity::data::EntityId,
) {
    match input {
        InputOrigin::Current(publication) => (
            &publication.origin.meaning,
            publication.origin.meaning.source_scope,
        ),
        InputOrigin::Applied(index) => {
            let meaning = parts[*index]
                .publication
                .producing_meaning()
                .expect("admitted earlier part");
            (meaning, meaning.source_scope)
        }
        InputOrigin::Restored { index, source } => (
            &binding
                .owners
                .source
                .restoration()
                .expect("issued restored input")
                .publications[*index]
                .origin
                .meaning,
            *source,
        ),
    }
}
/// Whether part `parent`'s latest round appended before part
/// `consumer`'s latest round. Without successor rounds this is part order;
/// a round with no batch orders by its append ordinal like any other.
fn round_precedes(parts: &[CoupledPublicationPart], parent: usize, consumer: usize) -> bool {
    parent != consumer
        && match (parts.get(parent), parts.get(consumer)) {
            (Some(parent), Some(consumer)) => parent.chain.latest < consumer.chain.latest,
            _ => false,
        }
}

pub(super) fn retain(
    binding: &BoundAuthoredCoupling,
    identity: &RelationalMutationProposalIdentity,
    parts: &mut [CoupledPublicationPart],
    execution: &ExecutionRequest<'_, '_>,
    issuance: Option<&Arc<InputIssuance>>,
    consumer: &RelationalRequiredOutputInput,
    parents: &[&RelationalRequiredOutputInput],
) -> Result<(), TransactionCommitError> {
    if consumer.proposal != *identity
        || !issuance.is_some_and(|issuance| Arc::ptr_eq(issuance, &consumer.issuance))
    {
        return Err(origin_denial("foreign consumed-output proposal"));
    }
    let index = match &consumer.origin {
        InputOrigin::Applied(index) => *index,
        InputOrigin::Current(_) | InputOrigin::Restored { .. } => return Ok(()),
    };
    let part = parts
        .get(index)
        .ok_or_else(|| origin_denial("consumed-output part is unavailable"))?;
    let own = part.publication.producing_meaning().expect("admitted part");
    let subject = own.source_scope;
    let bytes = execution
        .run_native_with_checkpoint(|context| {
            let mut bytes = parents
                .len()
                .checked_mul(std::mem::size_of::<RequiredOutputPublicationMeaning>())
                .ok_or_else(|| origin_denial("consumed-output size overflow"))?;
            for parent in parents {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if parent.proposal != *identity
                    || !Arc::ptr_eq(&parent.issuance, &consumer.issuance)
                {
                    return Err(reject(Cause::ForeignCoupledOwner, subject));
                }
                // The parent's latest round applied before the consumer's
                // latest round: a successor folds into its chain's part
                // index, so part order alone no longer orders rounds.
                if matches!(&parent.origin, InputOrigin::Applied(previous) if !round_precedes(parts, *previous, index)) {
                    return Err(reject(Cause::ForeignCoupledOwner, subject));
                }
                let (meaning, source) = input_route(parts, binding, &parent.origin);
                checkpoint(
                    context,
                    (own.family.len() + meaning.family.len() + 65) as u64,
                )
                .map_err(custody::execution_stop)?;
                if own.family == meaning.family && own.source_scope == source
                    && own.source_partition == meaning.source_partition {
                    return Err(reject(Cause::ForeignCoupledOwner, subject));
                }
                bytes = bytes.checked_add(meaning.family.len()).ok_or_else(|| {
                    origin_denial("consumed-output size overflow")
                })?;
            }
            Ok::<_, TransactionCommitError>(bytes)
        })
        .map_err(custody::execution_stop)?
        .0?;
    let memory = execution
        .reserve_retained_memory(bytes as u64)
        .map_err(custody::execution_stop)?;
    let retained = execution
        .run_native_with_checkpoint(|context| {
            let mut retained: Vec<RequiredOutputPublicationMeaning> = Vec::new();
            retained
                .try_reserve_exact(parents.len())
                .map_err(|_| origin_denial("consumed-output allocation unavailable"))?;
            for parent in parents {
                let (meaning, source) = input_route(parts, binding, &parent.origin);
                checkpoint(context, (meaning.family.len() + retained.len() + 65) as u64)
                    .map_err(custody::execution_stop)?;
                let mut found = None;
                for prior in &retained {
                    checkpoint(
                        context,
                        (prior.family.len() + meaning.family.len() + 65) as u64,
                    )
                    .map_err(custody::execution_stop)?;
                    if prior.family == meaning.family
                        && prior.source_scope == source
                        && prior.source_partition == meaning.source_partition
                    {
                        found = Some(prior);
                        break;
                    }
                }
                if let Some(prior) = found {
                    if prior.implementation != meaning.implementation
                        || prior.source_epoch != meaning.source_epoch
                    {
                        return Err(reject(Cause::ForeignCoupledOwner, subject));
                    }
                } else {
                    let mut meaning = meaning.clone();
                    meaning.source_scope = source;
                    retained.push(meaning);
                }
            }
            let comparisons = retained
                .len()
                .saturating_mul(usize::BITS as usize - retained.len().leading_zeros() as usize + 1);
            let width = retained
                .iter()
                .map(|input| input.family.len())
                .max()
                .unwrap_or(0)
                .saturating_add(65);
            checkpoint(context, comparisons.saturating_mul(width) as u64)
                .map_err(custody::execution_stop)?;
            retained.sort_by(|a, b| {
                (&a.family, a.source_scope, a.source_partition).cmp(&(
                    &b.family,
                    b.source_scope,
                    b.source_partition,
                ))
            });
            Ok(retained)
        })
        .map_err(custody::execution_stop)?
        .0?;
    let part = &mut parts[index];
    if let Some(prior) = &part.inputs {
        if prior.meanings != retained {
            return Err(reject(Cause::ForeignCoupledOwner, subject));
        }
    } else {
        part.inputs = Some(RequiredOutputInputs {
            meanings: retained,
            memory,
        });
    }
    Ok(())
}
