//! One native relation association court for retained before, candidate and cold images.
use super::{
    CanonicalCommitEnvelope, CanonicalRequiredOutputOrigin, CodecRetirementClosureStop as Stop,
};
use crate::branch::RelationalBranchRoot;
use crate::history::data::CommitId;
use crate::identity::data::{RelationId, VersionId};
use crate::storage::overlay::{
    current_relation_partition_source_for_slot, CurrentPartitionAccess, OverlayStateView,
    PartitionCloneMode, WorkingState,
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
mod creation;
mod endpoints;
mod incidence;
pub(super) use creation::validate_owned;

#[allow(clippy::too_many_arguments)]
pub(super) fn validate(
    root: &RelationalBranchRoot,
    candidate: Option<&WorkingState>,
    origin: &CanonicalRequiredOutputOrigin,
    current: Option<&CanonicalCommitEnvelope>,
    lookup_commit: &mut impl FnMut(CommitId) -> Option<Arc<CanonicalCommitEnvelope>>,
    lookup: &mut impl FnMut(VersionId) -> Option<Arc<CanonicalCommitEnvelope>>,
    maximum_hops: usize,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    let overlay;
    let state: &dyn CurrentPartitionAccess = match candidate {
        Some(candidate) => {
            overlay = OverlayStateView::new(root, candidate);
            &overlay
        }
        None => root,
    };
    let incidence: &dyn CurrentPartitionAccess =
        if candidate.is_some_and(|state| state.clone_mode == PartitionCloneMode::EntityOnly) {
            root
        } else {
            state
        };
    charge(context, origin.relations.len())?;
    if origin
        .relations
        .windows(2)
        .any(|pair| pair[0].id >= pair[1].id)
    {
        return Err("publication relation roster is not distinct canonical membership".into());
    }
    // Authenticate every claim before considering where its current endpoints moved.
    for entry in &origin.relations {
        charge(
            context,
            3 + 2 * (usize::BITS - origin.outputs.len().leading_zeros()) as usize,
        )?;
        if origin.outputs.binary_search(&entry.source).is_err()
            && origin.outputs.binary_search(&entry.target).is_err()
        {
            return Err("claimed publication relation has no output endpoint".into());
        }
        let (_, partition) = current_relation_partition_source_for_slot(
            state,
            entry.id.partition_id,
            entry.id.slot_index(),
        )
        .ok_or("publication relation partition absent")?;
        let row = partition
            .relation_arena()
            .get(&entry.id)
            .filter(|row| row.is_live())
            .ok_or("publication relation generation absent or retired")?;
        let write = partition
            .relation_arena()
            .current_metadata_at(entry.id.slot_index())
            .map(|m| m.effective_at);
        if row.kind_id() != Some(entry.kind)
            || !write
                .map(|write| {
                    super::output_materialization_write_matches(
                        &crate::transactions::data::RecordRef::Relation(entry.id),
                        origin.commit.commit_id,
                        entry.after_write,
                        write,
                        lookup_commit,
                        lookup,
                        maximum_hops,
                        context,
                    )
                })
                .transpose()?
                .unwrap_or(false)
            || !row
                .extra()
                .endpoints
                .as_ref()
                .is_some_and(|e| e.source == entry.source && e.target == entry.target)
        {
            return Err("incident relation breaks producer write/endpoints continuity".into());
        }
        if root
            .schema_authority()
            .relation_aspect_plan(entry.kind)
            .is_none()
        {
            return Err("publication relation installed physical contract absent".into());
        }
        // Warm callers carry authenticated predecessor/candidate provenance;
        // the cold caller has already run relation_payload for every claim.
        endpoints::validate(
            state,
            root.schema_authority().registry(),
            entry.kind,
            entry.source,
            entry.target,
            context,
        )?;
    }
    incidence::validate(
        incidence, state, root, origin, current, lookup, lease, context,
    )
}

pub(super) fn charge(context: &mut MapKernelContext<'_, '_>, work: usize) -> Result<(), Stop> {
    let work = u64::try_from(work).map_err(|_| {
        Stop::Request(crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)
    })?;
    context
        .checkpoint(work)
        .map_err(|cause| Stop::Request(cause.into()))
}

pub(super) fn writes(
    envelope: &CanonicalCommitEnvelope,
    id: RelationId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    charge(
        context,
        envelope.patch.authoritative_record_patches.len()
            + envelope.merged_plan.merged_intents.len(),
    )?;
    Ok(envelope
        .patch
        .authoritative_record_patches
        .iter()
        .any(|p| p.target == crate::transactions::data::RecordRef::Relation(id))
        || envelope.merged_plan.merged_intents.iter().any(|intent| {
            !matches!(
                intent,
                crate::transactions::data::MutationIntent::Relation(
                    crate::transactions::data::RelationMutationIntent::Revalidate(_)
                )
            ) && intent.existing_record_target()
                == Some(crate::transactions::data::ExistingRecordTarget::Relation(
                    id,
                ))
        }))
}
