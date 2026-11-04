//! Installed correction declarations select authored anchors once for admission.
use super::WorthQueryPrimaryGraphApplicationAttempt;
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
    if let Some(anchor) = attempt
        .aftermath_causality()
        .and_then(|causality| causality.prepared_correction().authored_retirement_anchor())
    {
        let prepared = attempt
            .aftermath_causality()
            .expect("selected correction anchor")
            .prepared_correction();
        return Ok(Some((anchor, prepared.authored_retirement_co_anchors())));
    }
    let identity = attempt
        .original_authority()
        .retained_governed_input_carrier()
        .and_then(|input| input.correction())
        .and_then(|original| original.association().declaration().authored_identity());
    let Some(identity) = identity else {
        return Ok(None);
    };
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
