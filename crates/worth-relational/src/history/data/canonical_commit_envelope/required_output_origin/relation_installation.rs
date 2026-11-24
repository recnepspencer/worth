//! Prior relation membership retains the predecessor's exact write fences.
use super::{origin_request, CanonicalCommitEnvelopeBody, CanonicalRequiredOutputRelation};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::MapKernelContext;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

/// Prior edges retain native before-write fences; only actual candidate changes
/// can update or retire them. New edges were captured from real allocations.
pub(super) fn continue_relations(
    envelope: &CanonicalCommitEnvelopeBody,
    history: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationHistoryRequest<'_, '_, '_>,
    prior: &super::CanonicalRequiredOutputOrigin,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    state: &crate::storage::overlay::WorkingState,
    relations: &mut Vec<CanonicalRequiredOutputRelation>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    use crate::publication::patch::data::RecordStructuralChange as Change;
    use crate::storage::overlay::{current_relation_partition_source_for_slot, OverlayStateView};
    use crate::transactions::data::primitives::RecordRef;
    let after_image = OverlayStateView::new((*basis.owner_inner()).root.as_ref(), state);
    for relation in &prior.relations {
        context
            .checkpoint(2 + envelope.patch.authoritative_record_patches.len() as u64)
            .map_err(origin_request)?;
        let before = (*basis.owner_inner())
            .root
            .get_current_partition(relation.id.partition_id)
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "prior relation partition absent",
                )
            })?;
        let row = before
            .relation_arena()
            .get(&relation.id)
            .filter(|row| row.is_live())
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "prior relation generation absent or retired",
                )
            })?;
        let write = before
            .relation_arena()
            .current_metadata_at(relation.id.slot_index())
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "prior relation write position absent",
                )
            })?
            .effective_at;
        if (write != relation.after_write
            && !crate::history::data::output_materialization_write_matches(
                &RecordRef::Relation(relation.id),
                prior.commit.commit_id,
                relation.after_write,
                write,
                &mut |commit| history.canonical_envelope(commit),
                &mut |version, context| history.canonical_envelope_for_version(version, context),
                history.canonical_catalog_len(),
                &history.execution(),
                context,
            )
            .map_err(super::installation::relation_stop)?)
            || row.kind_id() != Some(relation.kind)
            || !row.extra().endpoints.as_ref().is_some_and(|ends| {
                ends.source == relation.source && ends.target == relation.target
            })
        {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "relation write/endpoints break native producer continuity",
                ),
            );
        }
        let (_, after) = current_relation_partition_source_for_slot(
            &after_image,
            relation.id.partition_id,
            relation.id.slot_index(),
        )
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "successor relation partition absent",
            )
        })?;
        let row = after.relation_arena().get(&relation.id).ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "successor relation generation absent",
            )
        })?;
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
                return Err(
                    crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "relation retirement lacks this candidate's actual native operation",
                    ),
                );
            }
            continue;
        }
        let ends = row.extra().endpoints.as_ref().ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "successor relation endpoints absent",
            )
        })?;
        let after_write = after
            .relation_arena()
            .current_metadata_at(relation.id.slot_index())
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "successor relation write position absent",
                )
            })?
            .effective_at;
        if row.kind_id() != Some(relation.kind)
            || (after_write != write && after_write != envelope.commit.version_id)
            || patch.is_some_and(|patch| {
                patch.structural_change != Change::Updated || patch.contains_opaque_aspect
            })
        {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "successor relation differs from its actual native mutation",
                ),
            );
        }
        relations.push(CanonicalRequiredOutputRelation {
            id: relation.id,
            kind: relation.kind,
            source: ends.source,
            target: ends.target,
            birth: relation.birth,
            before_write: Some(write),
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
