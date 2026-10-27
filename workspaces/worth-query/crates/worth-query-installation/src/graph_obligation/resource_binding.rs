use super::{
    WorthQueryGraphObligationInstallationDenial, WorthQueryInstalledGraphCapabilityRequirement,
    WorthQueryInstalledGraphObligationContract as Contract,
    WorthQueryInstalledGraphObligationResourcePosture as Resources,
    WorthQueryInstalledGraphObligationSet,
};
use crate::{
    application_operation::WorthQueryInstalledAbilityRequirement,
    application_resource::WorthQueryResourceAccessPurpose,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

pub(crate) fn bind_resource_obligations(
    binding: &ApplicationSchemaBindingIdentity,
    name: &str,
    purpose: WorthQueryResourceAccessPurpose,
    ability: Option<&WorthQueryInstalledAbilityRequirement>,
    capabilities: &[WorthQueryInstalledGraphCapabilityRequirement],
) -> Result<WorthQueryInstalledGraphObligationSet, WorthQueryGraphObligationInstallationDenial> {
    let mut contracts = vec![Contract::PrincipalAuthorization];
    if let Some(requirement) = ability {
        contracts.push(Contract::AbilityAuthorization {
            requirements: vec![requirement.clone()],
        });
    }
    if !capabilities.is_empty() {
        contracts.push(Contract::CapabilityAuthorization {
            requirements: capabilities.to_vec(),
        });
    }
    let maximum_authorization_facts = 1usize
        .saturating_add(ability.map_or(0, |value| value.policy_paths().len().max(1)))
        .saturating_add(usize::from(!capabilities.is_empty()).saturating_mul(2));
    WorthQueryInstalledGraphObligationSet::for_resource(
        binding,
        name.to_owned(),
        purpose.name().to_owned(),
        contracts,
        Resources::ApplicationResource {
            maximum_authorization_facts,
        },
    )
    .map_err(Into::into)
}
