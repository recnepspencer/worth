//! Publication retirement and succession cannot sever a live owned source.
use super::*;
use crate::{
    publication::patch::data::RecordStructuralChange as Change, transactions::data::RecordRef,
};

pub(super) fn require_retained_sources(
    candidate: &CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[crate::mvcc::RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    if candidate.authored_cascade_retirement.is_some()
        || installed.source_scope_ownership.is_empty()
    {
        return Ok(());
    }
    context
        .checkpoint(
            candidate
                .patch
                .authoritative_record_patches
                .len()
                .saturating_mul(prior.members.len()) as u64,
        )
        .map_err(origin_request)?;
    context
        .checkpoint(roles.len() as u64)
        .map_err(origin_request)?;
    if roles
        .iter()
        .all(|role| role.posture == crate::mvcc::RequiredOutputRolePosture::Retire)
    {
        super::super::source_scope_ownership::require_cascade(prior, None)?;
    }
    let removed = prior
        .members
        .iter()
        .filter(|member| member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
        .filter(|member| {
            candidate
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Entity(member.id)
                        && patch.structural_change == Change::Deleted
                })
        })
        .map(|member| member.id)
        .collect::<Vec<_>>();
    if removed.is_empty() {
        return Ok(());
    }
    let selected = crate::history::data::select_owned_scope_dependent(
        &removed,
        candidate.commit.parents.first().copied(),
        runtime.history.canonical_catalog_len(),
        |commit| runtime.history.canonical_envelope(commit),
        context,
    )
    .map_err(super::relation_stop)?;
    if let Some(origin) = selected {
        return Err(TransactionCommitError::publication(crate::publication::data::PublicationError::owned_publication(
            crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
            }, origin.outputs[0], origin.meaning.source_scope)));
    }
    Ok(())
}
