//! Relation provenance is captured only from native allocations between fully
//! covered creates of this one installed output publication.
use super::payload_coverage::{validate_created_relation_payload, OutputPayloadCoverageFailure};
use super::{origin_request, CanonicalCommitEnvelope};
use crate::history::data::CodecRetirementClosureStop;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{
    CommitCreatedEntityBindings, CommitCreatedRelationBindings, EntityReference,
    TransactionCommitError,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
use worth_foundational::facade::AuthoritativeRecordAspectState;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalRequiredOutputRelation {
    pub(crate) id: RelationId,
    pub(crate) kind: KindId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
}

pub(super) fn capture_created_relations(
    envelope: &CanonicalCommitEnvelope,
    entities: &CommitCreatedEntityBindings,
    relations: &CommitCreatedRelationBindings,
    state: &crate::storage::overlay::WorkingState,
    outputs: &[EntityId],
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
    let mut captured = Vec::with_capacity(relations.iter().len());
    for (created, id) in relations.iter() {
        context.checkpoint(1).map_err(origin_request)?;
        let (EntityReference::Created(source), EntityReference::Created(target)) =
            (&created.source, &created.target)
        else {
            continue;
        };
        for endpoint in [source, target] {
            context
                .checkpoint(
                    entities
                        .resolve_work_bound(endpoint)
                        .ok_or_else(work_overflow)?,
                )
                .map_err(origin_request)?;
        }
        let (Some(source), Some(target)) = (entities.resolve(source), entities.resolve(target))
        else {
            return Err(crate::mvcc::origin_denial(
                "native relation create lost its allocated endpoints",
            ));
        };
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
            }),
            Err(OutputPayloadCoverageFailure::Unsupported) => continue,
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

impl CanonicalCommitEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn readmit_required_output_relation(
        &self,
        id: RelationId,
        kind: KindId,
        source: EntityId,
        target: EntityId,
        payload: Option<&AuthoritativeRecordAspectState>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), CodecRetirementClosureStop> {
        let origin = self
            .required_output_origin
            .as_ref()
            .ok_or("incident relation has no installed native output origin")?;
        context
            .checkpoint(u64::from(usize::BITS - origin.relations.len().leading_zeros()) + 1)
            .map_err(|cause| CodecRetirementClosureStop::Request(cause.into()))?;
        let entry = origin
            .relations
            .binary_search_by_key(&id, |relation| relation.id)
            .ok()
            .map(|position| &origin.relations[position])
            .ok_or("incident relation was not created inside the proven output closure")?;
        // Endpoint entity proofs have already readmitted this exact origin's
        // commit/basis against the same retained canonical envelope.
        if origin.commit != self.commit
            || entry.kind != kind
            || entry.source != source
            || entry.target != target
        {
            return Err("incident relation differs from its performed native create".into());
        }
        validate_created_relation_payload(self, id, payload, context).map_err(|failure| {
            match failure {
                OutputPayloadCoverageFailure::Request(cause) => {
                    CodecRetirementClosureStop::Request(cause)
                }
                OutputPayloadCoverageFailure::Unsupported => CodecRetirementClosureStop::Payload(
                    "incident relation has unsupported or unclassified present payload".into(),
                ),
                OutputPayloadCoverageFailure::Corrupt(detail) => {
                    CodecRetirementClosureStop::Payload(detail)
                }
            }
        })
    }
}

fn overflow() -> TransactionCommitError {
    origin_request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}

fn work_overflow() -> TransactionCommitError {
    origin_request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)
}
