use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use crate::application_query::{
    WorthQueryInstalledApplicationQueryAuthorization, WorthQueryInstalledApplicationQueryIdentity,
    WorthQueryInstalledGraphReadContract,
};

use super::{
    WorthQueryGraphObligationInstallationDenial, WorthQueryInstalledGraphCapabilityRequirement,
    WorthQueryInstalledGraphObligationContract, WorthQueryInstalledGraphObligationResourcePosture,
    WorthQueryInstalledGraphObligationSet,
};

pub(crate) fn bind_query_obligations(
    budget: worth_foundational::facade::CanonicalDigestWorkBudget,
    binding: &ApplicationSchemaBindingIdentity,
    name: &str,
    identity: &WorthQueryInstalledApplicationQueryIdentity,
    graph: &WorthQueryInstalledGraphReadContract,
    authorization: &WorthQueryInstalledApplicationQueryAuthorization,
    disclosure_capabilities: &[WorthQueryInstalledGraphCapabilityRequirement],
) -> Result<WorthQueryInstalledGraphObligationSet, WorthQueryGraphObligationInstallationDenial> {
    let maximum_authorization_facts = match authorization {
        WorthQueryInstalledApplicationQueryAuthorization::Public => 0,
        WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) => {
            requirement.policy_paths().len().max(1)
        }
    }
    .saturating_add(usize::from(!disclosure_capabilities.is_empty()).saturating_mul(2));
    let resources = WorthQueryInstalledGraphObligationResourcePosture::ApplicationQuery {
        maximum_traversal_depth: graph.maximum_traversal_depth(),
        maximum_result_count: graph.maximum_result_count(),
        maximum_authorization_facts,
    };
    let mut contracts = vec![WorthQueryInstalledGraphObligationContract::QueryGraphRead {
        graph: graph.clone(),
    }];
    if let WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) = authorization {
        contracts.push(
            WorthQueryInstalledGraphObligationContract::AbilityAuthorization {
                requirements: vec![requirement.clone()],
            },
        );
    }
    if !disclosure_capabilities.is_empty() {
        contracts.push(
            WorthQueryInstalledGraphObligationContract::CapabilityAuthorization {
                requirements: disclosure_capabilities.to_vec(),
            },
        );
    }
    WorthQueryInstalledGraphObligationSet::for_query(
        budget,
        binding,
        name.to_owned(),
        identity.clone(),
        contracts,
        resources,
    )
    .map_err(Into::into)
}
