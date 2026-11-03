//! Publication retirement and succession cannot sever a live owned source.
use super::*;
use crate::{
    identity::data::EntityId, publication::patch::data::RecordStructuralChange as Change,
    transactions::data::RecordRef,
};
use std::sync::Arc;

pub(super) struct RemovedOwningScopes {
    pub(super) anchors: Vec<EntityId>,
    pub(super) dependent: Arc<CanonicalRequiredOutputOrigin>,
}

pub(super) fn require_retained_sources(
    candidate: &CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[crate::mvcc::RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    if let Some(removal) =
        select_removed_owning_scopes(candidate, runtime, installed, prior, roles, context)?
    {
        super::super::source_scope_ownership::require_cascade(&removal.dependent, None)?;
    }
    Ok(())
}

/// Preserve the existing charged eligibility path for every non-cascading candidate.
pub(super) fn select_removed_owning_scopes(
    candidate: &CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[crate::mvcc::RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<RemovedOwningScopes>, TransactionCommitError> {
    if candidate.authored_cascade_retirement.is_some()
        || installed.source_scope_ownership.is_empty()
    {
        return Ok(None);
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
    let anchors = prior
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
    if anchors.is_empty() {
        return Ok(None);
    }
    let dependent = crate::history::data::select_owned_scope_dependent(
        &anchors,
        candidate.commit.parents.first().copied(),
        runtime.history.canonical_catalog_len(),
        |commit| runtime.history.canonical_envelope(commit),
        context,
    )
    .map_err(super::relation_stop)?;
    Ok(dependent.map(|dependent| RemovedOwningScopes { anchors, dependent }))
}
