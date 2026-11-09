//! Declared co-roles qualify inverse preimages by exact native record.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_relational::facade::transactions::{EntityReference, RecordRef};
pub(super) fn select(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<Option<Vec<(RecordRef, &str)>>, WorthQueryProviderSessionFailure> {
    if let Some(prepared) = attempt
        .aftermath_causality()
        .map(|causality| causality.prepared_correction())
        && let Some(anchor) = prepared.authored_retirement_anchor()
    {
        let co = prepared.authored_retirement_co_anchors();
        if !co.is_empty()
            || prepared.authored_declaration().is_some_and(|identity| {
                identity
                    .co_anchors()
                    .iter()
                    .any(|member| member.member_owner().is_some())
            })
        {
            let mut anchors = Vec::with_capacity(co.len() + 1);
            for id in std::iter::once(anchor).chain(co) {
                let entity = attempt
                    .native_publication_roles()
                    .find_map(|(_, posture, entity, target)| {
                        (posture == Posture::Retire && *target == EntityReference::Existing(id))
                            .then_some(entity)
                    })
                    .ok_or_else(failure)?;
                anchors.push((RecordRef::Entity(id), entity));
            }
            return Ok(Some(anchors));
        }
    }
    let identity = attempt
        .original_authority()
        .retained_governed_input_carrier()
        .and_then(|input| input.correction())
        .and_then(|original| original.association().declaration().authored_identity());
    let Some(identity) = identity.filter(|identity| {
        !identity.co_anchors().is_empty() && identity.parts().source_posture == Posture::Retire
    }) else {
        return Ok(None);
    };
    let mut anchors = Vec::with_capacity(identity.co_anchors().len() + 1);
    for declaration in std::iter::once(identity).chain(identity.co_anchors()) {
        if declaration.member_owner().is_some() {
            anchors.extend(attempt.native_publication_roles().filter_map(
                |(role, posture, entity, target)| match target {
                    EntityReference::Existing(id)
                        if role.starts_with(&declaration.parts().source_role)
                            && posture == Posture::Retire =>
                    {
                        Some((RecordRef::Entity(*id), entity))
                    }
                    _ => None,
                },
            ));
            continue;
        }
        let role = attempt
            .native_publication_roles()
            .find(|(name, posture, _, _)| {
                *name == declaration.parts().source_role && *posture == Posture::Retire
            });
        if let Some((_, _, entity, EntityReference::Existing(id))) = role {
            anchors.push((RecordRef::Entity(*id), entity));
        } else if !declaration.is_optional() {
            return Err(failure());
        }
    }
    Ok(Some(anchors))
}
fn failure() -> WorthQueryProviderSessionFailure {
    provider_failure(
        WorthQueryProviderSessionProtocolStage::Commit,
        "authored preimage omitted its declared anchor",
    )
}
