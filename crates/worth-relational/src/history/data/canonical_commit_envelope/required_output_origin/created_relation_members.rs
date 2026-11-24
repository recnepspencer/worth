//! Actual producer-created relations touch at least one complete output member.
//! Owning an edge never adopts its external endpoint entity.
use super::payload_coverage::{validate_created_relation_payload, OutputPayloadCoverageFailure};
use super::{origin_request, CanonicalCommitEnvelopeBody};
use crate::identity::data::EntityId;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::EntityReference;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

use super::CanonicalRequiredOutputRelation;

pub(in crate::history::data) fn capture_created_relations(
    envelope: &CanonicalCommitEnvelopeBody,
    entities: &CommitCreatedEntityBindings,
    relations: &CommitCreatedRelationBindings,
    state: &dyn CurrentPartitionAccess,
    outputs: &[EntityId],
    prior_count: usize,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    contribution_records: Option<&[crate::transactions::data::primitives::RecordRef]>,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<CanonicalRequiredOutputRelation>, TransactionCommitError> {
    let mut lookup_bytes = 0;
    for (created, _) in relations.iter() {
        context.checkpoint(1).map_err(origin_request)?;
        for endpoint in [&created.source, &created.target] {
            if let EntityReference::Created(endpoint) = endpoint {
                lookup_bytes =
                    lookup_bytes.max(endpoint.client_key.as_raw_str().map_or(0, str::len));
            }
        }
    }
    let _lookup_memory = ExecutionMemoryReservation::reserve_in_scope(
        lease,
        u64::try_from(lookup_bytes).map_err(|_| overflow())?,
    )
    .map_err(|stop| origin_request(crate::execution::denial::lease_denial(stop)))?;
    // The parent reserves this retained vector before either allocation.
    let mut captured = Vec::with_capacity(
        relations
            .iter()
            .len()
            .checked_add(prior_count)
            .ok_or_else(overflow)?,
    );
    for (created, id) in relations.iter() {
        context.checkpoint(1).map_err(origin_request)?;
        if let Some(records) = contribution_records {
            context
                .checkpoint(
                    records.len() as u64
                        + u64::from(
                            usize::BITS
                                - registry.native_private_kinds.len().max(1).leading_zeros(),
                        )
                        + 1,
                )
                .map_err(origin_request)?;
            if !records.contains(&crate::transactions::data::primitives::RecordRef::Relation(
                *id,
            )) && !registry.native_private_kinds.contains(&created.kind_id)
            {
                continue;
            }
        }
        let mut resolve = |reference: &EntityReference| match reference {
            EntityReference::Existing(id) => Ok(*id),
            EntityReference::Created(target) => {
                context
                    .checkpoint(
                        entities
                            .resolve_work_bound(target)
                            .ok_or_else(work_overflow)?,
                    )
                    .map_err(origin_request)?;
                entities.resolve(target).ok_or_else(|| {
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "new relation lost its actual allocated endpoint",
                    )
                })
            }
        };
        let source = resolve(&created.source)?;
        let target = resolve(&created.target)?;
        context
            .checkpoint(2 * u64::from(usize::BITS - outputs.len().leading_zeros()) + 2)
            .map_err(origin_request)?;
        if outputs.binary_search(&source).is_err() && outputs.binary_search(&target).is_err() {
            continue;
        }
        let record = state
            .get_current_partition(id.partition_id)
            .and_then(|partition| partition.relation_arena().get(id))
            .filter(|record| record.is_live())
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "native relation create is not present",
                )
            })?;
        if record.kind_id() != Some(created.kind_id)
            || !record
                .extra()
                .endpoints
                .as_ref()
                .is_some_and(|endpoints| endpoints.source == source && endpoints.target == target)
        {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "native relation allocation differs from its create",
                ),
            );
        }
        match validate_created_relation_payload(
            envelope,
            *id,
            record.extra().authoritative_aspect_state(),
            context,
        ) {
            Ok(()) => captured.push(CanonicalRequiredOutputRelation {
                id: *id,
                kind: created.kind_id,
                source,
                target,
                birth: envelope.commit.commit_id,
                before_write: None,
                after_write: envelope.commit.version_id,
            }),
            Err(OutputPayloadCoverageFailure::Unsupported(reason)) => {
                let mut error = crate::publication::data::PublicationError::new(
                    crate::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                    reason.relation_detail(envelope, *id, created.kind_id),
                );
                error.context = error.context.with_records(vec![
                    crate::transactions::data::primitives::RecordRef::Relation(*id),
                ]);
                return Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(error));
            }
            Err(OutputPayloadCoverageFailure::Request(cause)) => return Err(origin_request(cause)),
            Err(OutputPayloadCoverageFailure::Corrupt(detail)) => {
                return Err(
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        detail,
                    ),
                )
            }
        }
    }
    let sort_work = captured
        .len()
        .checked_mul((usize::BITS - captured.len().leading_zeros()) as usize + 1)
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(work_overflow)?;
    context.checkpoint(sort_work).map_err(origin_request)?;
    captured.sort_unstable_by_key(|relation| relation.id);
    Ok(captured)
}

fn overflow() -> TransactionCommitError {
    origin_request(
        crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}

fn work_overflow() -> TransactionCommitError {
    origin_request(
        crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
    )
}
