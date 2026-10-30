//! Rich materialization has its own versioned policy and actual original outcome identity.
use super::super::rich::WorthQueryApplicationRichMaterialization;
use super::*;
pub(in super::super) fn rich_identity<Stop>(
    reference: &str,
    body: &WorthQueryApplicationRichMaterialization,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    let families = super::super::super::inventory::CausalEvidenceFamily::ALL;
    let mut parts: [Part<'_>; super::super::super::inventory::CausalEvidenceFamily::ALL.len() + 5] =
        std::array::from_fn(|_| Part::Text("not_requested"));
    parts[0] = Part::Text(reference);
    parts[1] = Part::Text(body.policy().as_str());
    parts[2] = Part::Text(body.projection_label());
    parts[3] = Part::Text(body.source.label());
    parts[4] = Part::Text(body.producer_label());
    for (index, family) in families.into_iter().enumerate() {
        if body.request().requests(family) {
            parts[index + 5] = Part::Text(family.as_str());
        }
    }
    compose(
        "query.application-inspection.rich-materialization.v2",
        &parts,
        admit,
    )
}
