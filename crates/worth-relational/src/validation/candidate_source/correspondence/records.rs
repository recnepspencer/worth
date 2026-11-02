//! Compare the complete authoritative records, including absence and contracts.
use super::*;
use crate::branch::RelationalMaterializationRecord;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::RecordRef;
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(super) fn matches(
    draft: &AppliedMutationDraft,
    execution: ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    source: &super::super::NativeMaterializationSource,
    after: &InvariantStateView<'_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Denial> {
    let manifest = source.records();
    // The actual port's manifest is ordered by full record identity. Check the
    // invariant here too, before using it as a binary-search containment index.
    for pair in manifest.windows(2) {
        charge(context, 1)?;
        if record_ref(&pair[0]) >= record_ref(&pair[1]) {
            return Ok(false);
        }
    }
    for changed in &draft.outcome.effect.publication.changed_records {
        charge(
            context,
            u64::from(usize::BITS - manifest.len().leading_zeros()) + 1,
        )?;
        if manifest
            .binary_search_by(|entry| record_ref(entry).cmp(changed))
            .is_err()
        {
            return Ok(false);
        }
    }
    let backing = crate::branch::SymbolAdmittedRootRead::new(
        &source.basis().inner.root,
        &runtime.services.symbols,
    );
    let before = InvariantStateView::new(&backing, source.basis().observation().version_id());
    for entry in manifest {
        charge(context, 8)?;
        let partition = match entry {
            RelationalMaterializationRecord::Entity { entity_id, .. } => entity_id.partition_id,
            RelationalMaterializationRecord::Relation { relation_id, .. } => {
                relation_id.partition_id
            }
        };
        backing
            .demand_head_partition(&execution, partition)
            .map_err(|cause| ReadDenial::Request(cause.into()))?;
        let (left, right) = match *entry {
            RelationalMaterializationRecord::Entity { entity_id, kind_id } => {
                if ![&before, after].iter().all(|view| {
                    view.entity_metadata(entity_id)
                        .is_some_and(|row| row.entity_id == entity_id && row.kind_id == kind_id)
                }) {
                    return Ok(false);
                }
                (
                    before.entity_aspect_state(entity_id),
                    after.entity_aspect_state(entity_id),
                )
            }
            RelationalMaterializationRecord::Relation {
                relation_id,
                kind_id,
                source,
                target,
            } => {
                if ![&before, after].iter().all(|view| {
                    view.relation_metadata(relation_id).is_some_and(|row| {
                        row.relation_id == relation_id
                            && row.kind_id == kind_id
                            && row.source == source
                            && row.target == target
                    })
                }) {
                    return Ok(false);
                }
                (
                    before.relation_aspect_state(relation_id),
                    after.relation_aspect_state(relation_id),
                )
            }
        };
        if !aspects_match(left, right, context)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn record_ref(entry: &RelationalMaterializationRecord) -> RecordRef {
    match *entry {
        RelationalMaterializationRecord::Entity { entity_id, .. } => RecordRef::Entity(entity_id),
        RelationalMaterializationRecord::Relation { relation_id, .. } => {
            RecordRef::Relation(relation_id)
        }
    }
}

fn aspects_match(
    left: Option<&AuthoritativeRecordAspectState>,
    right: Option<&AuthoritativeRecordAspectState>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Denial> {
    let (left, right) = match (left, right) {
        (None, None) => return Ok(true),
        (Some(left), Some(right)) => (left.aspects(), right.aspects()),
        _ => return Ok(false),
    };
    if left.len() != right.len() {
        return Ok(false);
    }
    for ((left_key, left), (right_key, right)) in left.entries().zip(right.entries()) {
        let bytes = left_key
            .as_str()
            .len()
            .checked_add(right_key.as_str().len())
            .and_then(|bytes| bytes.checked_add(left.semantic_byte_width()))
            .and_then(|bytes| bytes.checked_add(right.semantic_byte_width()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        charge(context, bytes.checked_add(1).ok_or_else(overflow)?)?;
        if left_key != right_key || left != right {
            return Ok(false);
        }
    }
    Ok(true)
}
