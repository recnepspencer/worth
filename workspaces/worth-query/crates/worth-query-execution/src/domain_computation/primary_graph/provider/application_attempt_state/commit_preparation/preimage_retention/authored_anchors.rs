//! Declared co-roles qualify inverse preimages by exact native record.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_relational::facade::transactions::{EntityReference, RecordRef};
/// The exact records whose every demanded field is retained. Under a
/// live-owner declaration the attempt's other changed demanded fields are
/// retained beside them, and zero anchors is an exact selection.
pub(super) struct Selection<'a> {
    pub(super) anchors: Vec<(RecordRef, &'a str)>,
    pub(super) own_changes: bool,
}
pub(super) fn select(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<Option<Selection<'_>>, WorthQueryProviderSessionFailure> {
    if let Some(prepared) = attempt.recorded_reverse() {
        // A recorded reverse retains by the anchors its preparation recorded.
        // Its own binding's declaration never selects for it. A reapplied
        // source is the forward attempt below.
        let declaration = prepared.authored_declaration();
        let own_changes =
            declaration.is_some_and(|identity| identity.live_member_owner().is_some());
        let anchor = prepared.authored_retirement_anchor();
        let co = prepared.authored_retirement_co_anchors();
        let anchored = anchor.is_some()
            && (!co.is_empty()
                || declaration.is_some_and(|identity| {
                    identity
                        .co_anchors()
                        .iter()
                        .any(|member| member.member_owner().is_some())
                }));
        if !own_changes && !anchored {
            return Ok(None);
        }
        let mut anchors = Vec::with_capacity(co.len() + 1);
        for id in anchor.into_iter().chain(co) {
            let entity = attempt
                .native_publication_roles()
                .find_map(|(_, posture, entity, target)| {
                    (posture == Posture::Retire && *target == EntityReference::Existing(id))
                        .then_some(entity)
                })
                .ok_or_else(failure)?;
            anchors.push((RecordRef::Entity(id), entity));
        }
        return Ok(Some(Selection {
            anchors,
            own_changes,
        }));
    }
    let identity = attempt.authored_correction_identity();
    // Live-owner member families select every member the mutation retires:
    // zero members is an exact empty selection, never a fallback to the attempt.
    let Some(identity) = identity.filter(|identity| {
        identity.live_member_owner().is_some()
            || (!identity.co_anchors().is_empty()
                && identity.parts().source_posture == Posture::Retire)
    }) else {
        return Ok(None);
    };
    let mut anchors = Vec::with_capacity(identity.co_anchors().len() + 1);
    for declaration in std::iter::once(identity).chain(identity.co_anchors()) {
        if declaration.member_owner().is_some() || declaration.live_member_owner().is_some() {
            anchors.extend(attempt.native_publication_roles().filter_map(
                |(role, posture, entity, target)| {
                    match target {
                        EntityReference::Existing(id)
                            if declaration
                                .member_role_suffix(role, posture, entity, false)
                                .is_some()
                                && posture == Posture::Retire =>
                        {
                            Some((RecordRef::Entity(*id), entity))
                        }
                        _ => None,
                    }
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
    Ok(Some(Selection {
        anchors,
        own_changes: identity.live_member_owner().is_some(),
    }))
}
fn failure() -> WorthQueryProviderSessionFailure {
    provider_failure(
        WorthQueryProviderSessionProtocolStage::Commit,
        "authored preimage omitted its declared anchor",
    )
}
