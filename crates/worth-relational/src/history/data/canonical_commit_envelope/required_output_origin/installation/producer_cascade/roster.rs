//! Exact candidate closure excludes the generator's own continuing membership.
use super::*;
pub(super) fn require(
    candidate: &CanonicalCommitEnvelopeBody,
    prior: &CanonicalRequiredOutputOrigin,
    roles: &[RequiredOutputRoleTarget],
    selected: &[SelectedOutputPublication],
    latest: &[SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<RequiredOutputRoleTarget>, TransactionCommitError> {
    let mut root_roles = Vec::new();
    for (index, role) in roles.iter().enumerate() {
        context
            .checkpoint(
                (index + prior.members.len() + role.role.len() + role.entity_name.len() + 1) as u64,
            )
            .map_err(origin_request)?;
        if roles[..index].iter().any(|old| old.target == role.target) {
            let id = match role.target {
                EntityReference::Existing(id) => id,
                _ => prior.meaning.source_scope,
            };
            return Err(reject(Cause::RepeatedTarget, id));
        }
        let EntityReference::Existing(id) = role.target else {
            root_roles.push(role.clone());
            continue;
        };
        if prior
            .members
            .iter()
            .any(|member| member.id == id && member.posture != Posture::Retire)
        {
            root_roles.push(role.clone());
            continue;
        }
        let mut matched = false;
        for group in selected {
            context
                .checkpoint((group.origin.members.len() + role.entity_name.len()) as u64)
                .map_err(origin_request)?;
            matched |= group.origin.members.iter().any(|member| {
                member.id == id
                    && member.kind == role.kind
                    && member.entity_name == role.entity_name
                    && member.posture != Posture::Retire
            });
        }
        if !matched || role.posture != Posture::Retire {
            let mut cause = Cause::ExtraEntity;
            for group in latest {
                context
                    .checkpoint((group.origin.members.len() + selected.len()) as u64)
                    .map_err(origin_request)?;
                if group.origin.outputs.binary_search(&id).is_ok()
                    && !selected
                        .iter()
                        .any(|p| Arc::ptr_eq(&p.origin, &group.origin))
                {
                    cause = if group.origin.source_scope_owned {
                        Cause::DifferentSourcePublication
                    } else {
                        Cause::UnrelatedPublication
                    };
                    break;
                }
            }
            return Err(reject(cause, id));
        }
    }
    for group in selected {
        let mut supplied = 0;
        let mut missing = None;
        for member in &group.origin.members {
            if member.posture == Posture::Retire {
                continue;
            }
            context
                .checkpoint(
                    (roles.len() + candidate.patch.authoritative_record_patches.len()) as u64,
                )
                .map_err(origin_request)?;
            if roles
                .iter()
                .any(|role| role.target == EntityReference::Existing(member.id))
                && candidate
                    .patch
                    .authoritative_record_patches
                    .iter()
                    .any(|patch| {
                        patch.target == RecordRef::Entity(member.id)
                            && patch.structural_change == Change::Deleted
                    })
            {
                supplied += 1;
            } else {
                missing = missing.or(Some(member.id));
            }
        }
        if let Some(id) = missing {
            return Err(reject(
                if supplied == 0 {
                    Cause::MissingPublication
                } else {
                    Cause::MissingMember
                },
                id,
            ));
        }
        for edge in &group.origin.relations {
            context
                .checkpoint(candidate.patch.authoritative_record_patches.len() as u64)
                .map_err(origin_request)?;
            if !candidate
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Relation(edge.id)
                        && patch.structural_change == Change::Deleted
                })
            {
                return Err(TransactionCommitError::publication(
                    crate::publication::data::PublicationError::authored_retirement(
                        Cause::MissingMember,
                        RecordRef::Relation(edge.id),
                    ),
                ));
            }
        }
    }
    // A deletion cannot hide outside both the generator and the complete owned closure.
    for patch in &candidate.patch.authoritative_record_patches {
        if let RecordRef::Entity(id) = patch.target
            && patch.structural_change == Change::Deleted
        {
            context
                .checkpoint(roles.len() as u64)
                .map_err(origin_request)?;
            if !roles.iter().any(|role| {
                role.target == EntityReference::Existing(id) && role.posture == Posture::Retire
            }) {
                return Err(reject(Cause::ExtraEntity, id));
            }
        }
    }
    Ok(root_roles)
}
