//! Publication retirement and succession cannot sever a live owned source.
use super::*;
use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange as Change;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;

pub(super) struct RemovedOwningScopes {
    pub(super) anchors: Vec<EntityId>,
    pub(super) dependent: Arc<CanonicalRequiredOutputOrigin>,
}

pub(super) fn require_retained_sources(
    candidate: &CanonicalCommitEnvelopeBody,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::OutputPreparationRequest<'_, '_, '_>,
    installed: &crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget],
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
    candidate: &CanonicalCommitEnvelopeBody,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::OutputPreparationRequest<'_, '_, '_>,
    installed: &crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<RemovedOwningScopes>, TransactionCommitError> {
    if (*candidate.owner_authored_cascade_retirement()).is_some()
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
        .all(|role| role.posture == crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        super::super::source_scope_ownership::require_cascade(prior, None)?;
    }
    let anchors = prior
        .members
        .iter()
        .filter(|member| member.posture != crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
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
        crate::history::data::OwnedScopeDependentSelection::RemovedSources(&anchors),
        runtime
            .current_ownership
            .as_deref()
            .ok_or_else(|| {
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "owning-source removal requires Native current ownership storage",
                )
            })?
            .origins(),
        context,
    )
    .map_err(|failure| super::relation_stop(failure.stop))?;
    Ok(dependent.map(|dependent| RemovedOwningScopes { anchors, dependent }))
}
