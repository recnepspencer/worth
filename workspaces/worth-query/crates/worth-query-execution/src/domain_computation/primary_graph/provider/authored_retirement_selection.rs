//! Installed correction declarations select authored anchors once for admission.
use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture as Posture;
use worth_relational::facade::{mvcc::CompanionPreflightStop, transactions::EntityReference};

/// The installed correction declaration chooses the role. Roster order and
/// reference reachability never choose an authored source.
pub(super) fn authored_anchor(
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<
    Option<(
        worth_relational::facade::identity::EntityId,
        Vec<worth_relational::facade::identity::EntityId>,
    )>,
    CompanionPreflightStop,
> {
    if let Some(prepared) = attempt.recorded_reverse() {
        return Ok(prepared
            .authored_retirement_anchor()
            .map(|anchor| (anchor, prepared.authored_retirement_co_anchors())));
    }
    let identity = attempt.authored_correction_identity();
    let Some(identity) = identity else {
        return Ok(None);
    };
    if identity.live_member_owner().is_some() {
        // Every member this mutation retires under a live owner is its own
        // authored source. Retiring none leaves an ordinary publication.
        let mut members =
            attempt
                .native_publication_roles()
                .filter_map(|(role, posture, entity, target)| match target {
                    EntityReference::Existing(id)
                        if posture == Posture::Retire
                            && std::iter::once(identity).chain(identity.co_anchors()).any(
                                |family| {
                                    family
                                        .member_role_suffix(role, posture, entity, false)
                                        .is_some()
                                },
                            ) =>
                    {
                        Some(*id)
                    }
                    _ => None,
                });
        let Some(anchor) = members.next() else {
            return Ok(None);
        };
        return Ok(Some((anchor, members.collect())));
    }
    let parts = identity.parts();
    if parts.source_posture != Posture::Retire {
        return Ok(None);
    }
    let mut matches = attempt
        .native_publication_roles()
        .filter(|(role, posture, _, _)| *role == parts.source_role && *posture == Posture::Retire);
    let target = matches.next().and_then(|(_, _, _, target)| match target {
        EntityReference::Existing(id) => Some(*id),
        _ => None,
    });
    if target.is_none() || matches.next().is_some() {
        return Err(CompanionPreflightStop::SelectedSourceMismatch);
    }
    let mut co_anchors = Vec::with_capacity(identity.co_anchors().len());
    for declaration in identity.co_anchors() {
        if declaration.member_owner().is_some() {
            co_anchors.extend(attempt.native_publication_roles().filter_map(
                |(role, posture, entity, target)| {
                    match target {
                        EntityReference::Existing(id)
                            if declaration
                                .member_role_suffix(role, posture, entity, false)
                                .is_some()
                                && posture == Posture::Retire =>
                        {
                            Some(*id)
                        }
                        _ => None,
                    }
                },
            ));
            continue;
        }
        let mut matched = attempt
            .native_publication_roles()
            .filter(|(role, posture, _, _)| {
                *role == declaration.parts().source_role && *posture == Posture::Retire
            });
        let co_anchor = matched.next().and_then(|(_, _, _, target)| match target {
            EntityReference::Existing(id) => Some(*id),
            _ => None,
        });
        if matched.next().is_some() || (co_anchor.is_none() && !declaration.is_optional()) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        if let Some(id) = co_anchor {
            co_anchors.push(id);
        }
    }
    Ok(target.map(|anchor| (anchor, co_anchors)))
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    /// The prepared inverse this attempt publishes. Only a recorded reverse has
    /// one: a reapplied source runs its own binding again, so it selects and
    /// retains authored anchors as the forward attempt it is.
    pub(in crate::domain_computation::primary_graph) fn recorded_reverse(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource> {
        self.aftermath_causality()
            .filter(|pending| pending.role() == WorthQueryAftermathCausalRole::RecordedReverse)
            .map(|pending| pending.prepared_correction())
    }

    /// A recorded reverse answers with its prepared declaration and is never
    /// read as the forward source of its own binding.
    pub(in crate::domain_computation::primary_graph) fn authored_correction_identity(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_aftermath::DeclaredAuthoredCorrection>
    {
        match self.recorded_reverse() {
            Some(prepared) => prepared.authored_declaration(),
            None => self
                .original_authority()
                .retained_governed_input_carrier()
                .and_then(|input| input.correction())
                .and_then(|original| original.association().declaration().authored_identity()),
        }
    }
}
