//! Rich materialization has its own versioned policy and actual original outcome identity.
use super::super::rich::WorthQueryApplicationRichMaterialization;
use super::*;
pub(in super::super) fn rich_identity<Stop>(
    reference: &super::super::report::InspectionReportPayload,
    body: &WorthQueryApplicationRichMaterialization,
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<BridgeIdentityEvidence, InspectionIdentityStop<Stop>> {
    let families = super::super::super::inventory::CausalEvidenceFamily::ALL;
    let mut parts: [Part<'_>; super::super::super::inventory::CausalEvidenceFamily::ALL.len() + 3] =
        std::array::from_fn(|_| Part::Text("not_requested"));
    parts[0] = Part::Text(reference.report_identity());
    parts[1] = Part::Text(body.policy().as_str());
    parts[2] = Part::Text(body.projection_label());
    for (index, family) in families.into_iter().enumerate() {
        if body.request().requests(family) {
            parts[index + 3] = Part::Text(family.as_str());
        }
    }
    compose(
        "query.application-inspection.rich-materialization.v1",
        &parts,
        admit,
    )
}
