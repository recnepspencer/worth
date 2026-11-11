//! Heterogeneous inverse fields require an authored anchor or its declared live owner.
use super::{WorthQueryAftermathInstallationDenial, WorthQueryAftermathInstallationDenialKind};
use worth_query_declaration::facade::{
    application_aftermath::{PortableApplicationAftermathContract, PortableCorrectionMechanism},
    application_schema::ApplicationSchemaMember,
};
pub(super) fn validate(
    portable: &PortableApplicationAftermathContract,
) -> Result<(), WorthQueryAftermathInstallationDenial> {
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = portable.mechanism() else {
        return Ok(());
    };
    let demand = inverse.preimage_demand();
    let Some(first) = demand.loci().first() else {
        return Ok(());
    };
    if demand
        .loci()
        .iter()
        .all(|locus| locus.entity() == first.entity())
    {
        return Ok(());
    }
    let authored = portable
        .correction_binding()
        .and_then(|binding| binding.authored_identity());
    for locus in demand.loci() {
        if !authored.is_some_and(|identity| std::iter::once(identity).chain(identity.co_anchors())
            .any(|anchor| matches!(anchor.parts().logical_key.as_ref(), ApplicationSchemaMember::Field { entity, .. } if entity == locus.entity())
                || anchor.live_member_owner().is_some_and(|owner|
                    owner.to() == locus.entity() && demand.relations().contains(owner)))) {
            return Err(WorthQueryAftermathInstallationDenial::new(WorthQueryAftermathInstallationDenialKind::PreImageDemandNotCoveredByAuthoredAnchors, locus.entity()));
        }
    }
    Ok(())
}
