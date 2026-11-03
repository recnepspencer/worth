//! Compare the submitted roster only after Native independently selected its closure.
use super::*;
use std::sync::Arc;

pub(super) fn require_roster(
    anchor: EntityId,
    roles: &[RequiredOutputRoleTarget],
    selected: &[SelectedOutputPublication],
    latest: &[SelectedOutputPublication],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    for (index, role) in roles.iter().enumerate() {
        let EntityReference::Existing(id) = role.target else {
            return Err(reject(Cause::ExtraEntity, anchor));
        };
        charge(context, index + 1).map_err(stopped)?;
        if roles[..index]
            .iter()
            .any(|prior| prior.target == role.target)
        {
            return Err(reject(Cause::RepeatedTarget, id));
        }
        if role.posture != Posture::Retire {
            return Err(reject(Cause::ExtraEntity, id));
        }
        if id == anchor {
            continue;
        }
        let mut matched = false;
        for envelope in selected {
            let origin = &envelope.origin;
            charge(context, origin.members.len() + role.entity_name.len()).map_err(stopped)?;
            matched |= origin.members.iter().any(|member| {
                member.id == id
                    && member.kind == role.kind
                    && member.entity_name == role.entity_name
                    && member.posture != Posture::Retire
            });
        }
        if !matched {
            for envelope in latest {
                let origin = &envelope.origin;
                charge(context, origin.members.len() + selected.len()).map_err(stopped)?;
                if origin.members.iter().any(|member| member.id == id) {
                    let in_selected = selected
                        .iter()
                        .any(|prior| Arc::ptr_eq(&prior.origin, &envelope.origin));
                    return Err(reject(
                        if in_selected {
                            Cause::ExtraEntity
                        } else if origin.source_scope_owned {
                            Cause::DifferentSourcePublication
                        } else {
                            Cause::UnrelatedPublication
                        },
                        id,
                    ));
                }
            }
            return Err(reject(Cause::ExtraEntity, id));
        }
    }
    charge(context, roles.len()).map_err(stopped)?;
    if !roles
        .iter()
        .any(|role| role.target == EntityReference::Existing(anchor))
    {
        return Err(reject(Cause::MissingMember, anchor));
    }
    for envelope in selected {
        let origin = &envelope.origin;
        let mut supplied = 0;
        let mut missing = None;
        for member in &origin.members {
            if member.posture == Posture::Retire {
                continue;
            }
            charge(context, roles.len()).map_err(stopped)?;
            if roles
                .iter()
                .any(|role| role.target == EntityReference::Existing(member.id))
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
    }
    Ok(())
}
