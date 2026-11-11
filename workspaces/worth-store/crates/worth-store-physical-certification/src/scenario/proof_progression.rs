use worth_foundational::CanonicalDigestId;
use worth_proof::{
    AdmitRecipeTransition, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    ContextualTransition, LowerRecipeTransition, Recipe, RecipeResolutionContext,
    ResolveRecipeTransition, Transition, Unresolved,
};

use super::authority::PhysicalScenarioAuthorityWitness;
use super::certified::CertifiedPhysicalScenario;
use super::definition::PhysicalSimulationScenarioDefinition;
use super::denial::PhysicalScenarioDefinitionDenial;

pub fn reject_raw_json_scenario_authority_attempt(
    _raw_json_document: &str,
) -> Result<CertifiedPhysicalScenario, PhysicalScenarioDefinitionDenial> {
    Err(PhysicalScenarioDefinitionDenial::JsonScenarioAuthorityDenied)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalScenarioResolutionBasis {
    digest: CanonicalDigestId,
    canonical_basis_entry_count: u32,
}

pub(crate) fn certify_scenario_definition(
    definition: PhysicalSimulationScenarioDefinition,
) -> Result<CertifiedPhysicalScenario, PhysicalScenarioDefinitionDenial> {
    let identity = definition.canonical_identity();
    let identity = identity?;
    let basis = PhysicalScenarioResolutionBasis {
        digest: CanonicalDigestId::new(*identity.digest_bytes()),
        canonical_basis_entry_count: identity.canonical_basis_entry_count(),
    };
    require_resolution_basis(&basis)?;
    let unresolved = Recipe::<Unresolved, _>::new(definition);
    let resolved = ResolveRecipeTransition
        .transition(
            unresolved,
            RecipeResolutionContext::new(
                basis,
                AuthorityWitness::from_authority_marker(PhysicalScenarioResolutionAuthority),
            ),
        )
        .into_value();
    let lowered = LowerRecipeTransition::new(CapabilityWitness::from_capability_marker(
        PhysicalScenarioLoweringCapability,
    ))
    .transition(resolved)
    .into_value();
    let admitted = AdmitRecipeTransition::new(AuthorityWitness::from_authority_marker(
        PhysicalScenarioAdmissionAuthority,
    ))
    .transition(lowered)
    .into_value();
    let admitted_definition = admitted.into_parts().0;
    Ok(CertifiedPhysicalScenario::from_admitted_definition(
        admitted_definition,
        identity,
        PhysicalScenarioAuthorityWitness::from_store_scenario_authority(),
    ))
}

fn require_resolution_basis(
    basis: &PhysicalScenarioResolutionBasis,
) -> Result<(), PhysicalScenarioDefinitionDenial> {
    if *basis.digest.bytes() == [0; 32] || basis.canonical_basis_entry_count == 0 {
        return Err(PhysicalScenarioDefinitionDenial::ProofProgressionSkipped);
    }
    Ok(())
}

struct PhysicalScenarioResolutionAuthority;
impl AuthorityMarker for PhysicalScenarioResolutionAuthority {}

struct PhysicalScenarioLoweringCapability;
impl CapabilityMarker for PhysicalScenarioLoweringCapability {}

struct PhysicalScenarioAdmissionAuthority;
impl AuthorityMarker for PhysicalScenarioAdmissionAuthority {}
