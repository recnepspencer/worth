//! Rich materialization has its own versioned policy and actual original outcome identity.
use super::super::rich::WorthQueryApplicationRichMaterialization;
use super::*;
pub(in super::super) fn rich_identity<Stop>(
    reference: &str,
    body: &WorthQueryApplicationRichMaterialization,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    let history = super::history::history_identity(
        reference,
        body.request().requests_conditional_history(),
        &body.history,
        admit,
    )?;
    let families = super::super::super::inventory::CausalEvidenceFamily::ALL;
    let mut parts: [Part<'_>; super::super::super::inventory::CausalEvidenceFamily::ALL.len() + 6] =
        std::array::from_fn(|_| Part::Text("not_requested"));
    parts[0] = Part::Text(reference);
    parts[1] = Part::Text(body.policy().as_str());
    parts[2] = Part::Text(body.projection_label());
    parts[3] = Part::Text(body.source.label());
    parts[4] = Part::Text(body.producer_label());
    parts[5] = Part::Text(history.terminal_projection_for_reporting());
    for (index, family) in families.into_iter().enumerate() {
        if body.request().requests(family) {
            parts[index + 6] = Part::Text(family.as_str());
        }
    }
    compose(
        "query.application-inspection.rich-materialization.v3",
        &parts,
        admit,
    )
}
