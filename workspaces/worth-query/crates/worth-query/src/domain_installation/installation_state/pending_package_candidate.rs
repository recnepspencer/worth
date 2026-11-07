use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::any::TypeId;

use crate::application::WorthQueryDomainEntryMarker;
use crate::runtime::WorthQueryInstalledDomainSubstrateProvenance;

use super::{WorthQueryAdmittedDomainPackage, WorthQueryDomainSemanticVersion};

pub(super) struct WorthQueryPendingPackageCandidate {
    pub(super) marker_type: TypeId,
    pub(super) package_identity: String,
    pub(super) domain_owner: String,
    pub(super) semantic_version: WorthQueryDomainSemanticVersion,
    pub(super) substrate_provenance: WorthQueryInstalledDomainSubstrateProvenance,
    pub(super) invariant_slots: Vec<String>,
    pub(super) declaration_family_slots: Vec<String>,
}

pub(super) fn classify_pending_package<D: WorthQueryDomainEntryMarker + 'static>(
    package: &WorthQueryAdmittedDomainPackage<D>,
) -> Result<WorthQueryPendingPackageCandidate, IdentityDerivationDenial> {
    let domain_owner = package.identity.canonical_owner();
    let semantic_version = package.identity.semantic_version();
    let package_identity = package.package_identity.as_str().to_string();
    let substrate_provenance = WorthQueryInstalledDomainSubstrateProvenance::new(
        domain_owner.clone(),
        semantic_version.major(),
        semantic_version.minor(),
        package_identity.clone(),
    )?;
    let invariant_slots = package
        .invariant_definitions
        .iter()
        .map(|definition| format!("{}:{}", domain_owner, definition.slot_key()))
        .collect();
    let declaration_family_slots = package
        .declaration_families
        .iter()
        .map(|definition| format!("{}:{}", domain_owner, definition.slot_key()))
        .collect();
    Ok(WorthQueryPendingPackageCandidate {
        marker_type: TypeId::of::<D>(),
        package_identity,
        domain_owner,
        semantic_version,
        substrate_provenance,
        invariant_slots,
        declaration_family_slots,
    })
}
