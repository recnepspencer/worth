//! Inspection observes original evidence under an independent metadata obligation.
use super::{
    identity::OBLIGATION_IDENTITY_BUDGET, WorthQueryGraphObligationInstallationDenial as Denial,
    WorthQueryInstalledGraphCapabilityRequirement,
    WorthQueryInstalledGraphObligationContract as Contract,
    WorthQueryInstalledGraphObligationResourcePosture as Resources,
    WorthQueryInstalledGraphObligationSet,
};
use crate::{
    application_query::WorthQueryInstalledApplicationQueryAuthorization,
    canonical_digest_derivation::InstallationCanonicalIdentityBasis,
};
use worth_query_declaration::facade::{
    application_query::ErasedApplicationQueryInspectionDefinition,
    application_schema::ApplicationSchemaBindingIdentity,
};

const PURPOSE: &str = "original-query-observation-metadata-v1";

pub(crate) fn bind_inspection_obligations(
    binding: &ApplicationSchemaBindingIdentity,
    name: &str,
    definition: &ErasedApplicationQueryInspectionDefinition,
    authorization: &WorthQueryInstalledApplicationQueryAuthorization,
    capabilities: &[WorthQueryInstalledGraphCapabilityRequirement],
) -> Result<WorthQueryInstalledGraphObligationSet, Denial> {
    let mut basis = InstallationCanonicalIdentityBasis::new(
        "worth-query.application-inspection-meaning",
        "worth-query-application-inspection-meaning-v1",
        OBLIGATION_IDENTITY_BUDGET,
    );
    basis.text("purpose", PURPOSE)?;
    basis.embedded_basis("inspection", definition.canonical_basis().basis())?;
    let (meaning, work) = basis.derive()?;
    let mut contracts = vec![Contract::PrincipalAuthorization];
    let ability_facts = match authorization {
        WorthQueryInstalledApplicationQueryAuthorization::Public => 0,
        WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) => {
            contracts.push(Contract::AbilityAuthorization {
                requirements: vec![requirement.clone()],
            });
            requirement.policy_paths().len().max(1)
        }
    };
    if !capabilities.is_empty() {
        contracts.push(Contract::CapabilityAuthorization {
            requirements: capabilities.to_vec(),
        });
    }
    // Existing policy-fact capacity convention, with explicit overflow refusal.
    let maximum_authorization_facts = 1usize
        .checked_add(ability_facts)
        .and_then(|n| n.checked_add(usize::from(!capabilities.is_empty()) * 2))
        .ok_or(Denial::InvalidContract)?;
    WorthQueryInstalledGraphObligationSet::for_inspection(
        binding,
        name.to_owned(),
        PURPOSE.to_owned(),
        contracts,
        Resources::ApplicationInspection {
            meaning,
            maximum_authorization_facts,
        },
        work,
    )
    .map_err(Into::into)
}
