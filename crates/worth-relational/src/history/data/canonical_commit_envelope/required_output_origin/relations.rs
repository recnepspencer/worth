//! Relation provenance is captured only from native allocations between fully
//! covered creates of this one installed output publication.
use super::payload_coverage::{validate_created_relation_payload, OutputPayloadCoverageFailure};
use super::{origin_request, CanonicalCommitEnvelope};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{
    CommitCreatedEntityBindings, CommitCreatedRelationBindings, EntityReference,
    TransactionCommitError,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalRequiredOutputRelation {
    pub(crate) id: RelationId,
    pub(crate) kind: KindId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    pub(crate) birth: crate::history::data::CommitId,
    pub(crate) before_write: Option<crate::identity::data::VersionId>,
    pub(crate) after_write: crate::identity::data::VersionId,
}

pub(super) fn capture_created_relations(
    envelope: &CanonicalCommitEnvelope,
    entities: &CommitCreatedEntityBindings,
    relations: &CommitCreatedRelationBindings,
    state: &crate::storage::overlay::WorkingState,
    outputs: &[EntityId],
    prior_count: usize,
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
                    crate::mvcc::origin_denial("new relation lost its actual allocated endpoint")
                })
            }
        };
        let source = resolve(&created.source)?;
        let target = resolve(&created.target)?;
        context
            .checkpoint(2 * u64::from(usize::BITS - outputs.len().leading_zeros()) + 2)
            .map_err(origin_request)?;
        if outputs.binary_search(&source).is_err() || outputs.binary_search(&target).is_err() {
            continue;
        }
        let record = state
            .partitions
            .get(&id.partition_id)
            .and_then(|partition| partition.relation_arena.get(id))
            .filter(|record| record.is_live())
            .ok_or_else(|| crate::mvcc::origin_denial("native relation create is not present"))?;
        if record.kind_id() != Some(created.kind_id)
            || !record
                .extra()
                .endpoints
                .as_ref()
                .is_some_and(|endpoints| endpoints.source == source && endpoints.target == target)
        {
            return Err(crate::mvcc::origin_denial(
                "native relation allocation differs from its create",
            ));
        }
        match validate_created_relation_payload(
            envelope,
            *id,
            record.extra().authoritative_aspect_state.as_ref(),
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
                    crate::publication::bundle::PublicationStage::BundleAssembly,
                    reason.relation_detail(envelope, *id, created.kind_id),
                );
                error.context = error
                    .context
                    .with_records(vec![crate::transactions::data::RecordRef::Relation(*id)]);
                return Err(crate::transactions::data::TransactionCommitError::publication(error));
            }
            Err(OutputPayloadCoverageFailure::Request(cause)) => return Err(origin_request(cause)),
            Err(OutputPayloadCoverageFailure::Corrupt(detail)) => {
                return Err(crate::mvcc::origin_denial(detail))
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
    origin_request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}

fn work_overflow() -> TransactionCommitError {
    origin_request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)
}

/// Prior edges retain native before-write fences; only actual candidate changes
/// can update or retire them. New edges were captured from real allocations.
pub(super) fn continue_relations(
    envelope: &CanonicalCommitEnvelope,
    prior: &super::CanonicalRequiredOutputOrigin,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    state: &crate::storage::overlay::WorkingState,
    relations: &mut Vec<CanonicalRequiredOutputRelation>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    use crate::storage::overlay::{relation_partition_source_for_slot, OverlayStateView};
    use crate::transactions::data::RecordRef;
    let after_image = OverlayStateView::new(basis.inner.root.as_ref(), state);
    for relation in &prior.relations {
        context
            .checkpoint(2 + envelope.patch.authoritative_record_patches.len() as u64)
            .map_err(origin_request)?;
        let before = basis
            .inner
            .root
            .partition_state(relation.id.partition_id)
            .ok_or_else(|| crate::mvcc::origin_denial("prior relation partition absent"))?;
        let row = before
            .relation_arena
            .get(&relation.id)
            .filter(|row| row.is_live())
            .ok_or_else(|| {
                crate::mvcc::origin_denial("prior relation generation absent or retired")
            })?;
        let write = before
            .relation_arena
            .metadata_history_at(relation.id.slot_index())
            .and_then(|rows| rows.get(rows.len().checked_sub(1)?))
            .ok_or_else(|| crate::mvcc::origin_denial("prior relation write position absent"))?
            .effective_at;
        if write != relation.after_write
            || row.kind_id() != Some(relation.kind)
            || !row.extra().endpoints.as_ref().is_some_and(|ends| {
                ends.source == relation.source && ends.target == relation.target
            })
        {
            return Err(crate::mvcc::origin_denial(
                "relation write/endpoints break native producer continuity",
            ));
        }
        let (_, after) = relation_partition_source_for_slot(
            &after_image,
            relation.id.partition_id,
            relation.id.slot_index(),
        )
        .ok_or_else(|| crate::mvcc::origin_denial("successor relation partition absent"))?;
        let row = after
            .relation_arena
            .get(&relation.id)
            .ok_or_else(|| crate::mvcc::origin_denial("successor relation generation absent"))?;
        let patch = envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Relation(relation.id));
        if !row.is_live() {
            if row.retired_at() != Some(envelope.commit.version_id)
                || patch.is_none_or(|patch| {
                    !matches!(
                        patch.structural_change,
                        Change::Deleted | Change::RetainedForAudit
                    )
                })
            {
                return Err(crate::mvcc::origin_denial(
                    "relation retirement lacks this candidate's actual native operation",
                ));
            }
            continue;
        }
        let ends = row
            .extra()
            .endpoints
            .as_ref()
            .ok_or_else(|| crate::mvcc::origin_denial("successor relation endpoints absent"))?;
        let after_write = after
            .relation_arena
            .metadata_history_at(relation.id.slot_index())
            .and_then(|rows| rows.get(rows.len().checked_sub(1)?))
            .ok_or_else(|| crate::mvcc::origin_denial("successor relation write position absent"))?
            .effective_at;
        if row.kind_id() != Some(relation.kind)
            || (after_write != relation.after_write && after_write != envelope.commit.version_id)
            || patch.is_some_and(|patch| patch.structural_change != Change::Updated)
        {
            return Err(crate::mvcc::origin_denial(
                "successor relation differs from its actual native mutation",
            ));
        }
        relations.push(CanonicalRequiredOutputRelation {
            id: relation.id,
            kind: relation.kind,
            source: ends.source,
            target: ends.target,
            birth: relation.birth,
            before_write: Some(relation.after_write),
            after_write,
        });
    }
    context
        .checkpoint(
            (relations.len() * ((usize::BITS - relations.len().leading_zeros()) as usize + 1))
                as u64,
        )
        .map_err(origin_request)?;
    relations.sort_unstable_by_key(|relation| relation.id);
    Ok(())
}
