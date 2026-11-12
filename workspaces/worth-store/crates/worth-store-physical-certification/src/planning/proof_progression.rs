use worth_foundational::CanonicalDigestId;
use worth_proof::{
    AdmitRecipeTransition, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    ContextualTransition, LowerRecipeTransition, Recipe, RecipeResolutionContext,
    ResolveRecipeTransition, Transition, Unresolved,
};

use super::{PhysicalSimulationPlan, SimulationPlanDenial};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimulationPlanResolutionBasis {
    digest: CanonicalDigestId,
    canonical_basis_entry_count: u32,
}

pub fn reject_unresolved_simulation_plan_recipe(
    _recipe: Recipe<Unresolved, PhysicalSimulationPlan>,
) -> Result<PhysicalSimulationPlan, SimulationPlanDenial> {
    Err(SimulationPlanDenial::ProofProgressionSkipped)
}

pub(crate) fn admit_simulation_plan(
    plan: PhysicalSimulationPlan,
) -> Result<PhysicalSimulationPlan, SimulationPlanDenial> {
    let basis = SimulationPlanResolutionBasis {
        digest: CanonicalDigestId::new(*plan.identity().digest_bytes()),
        canonical_basis_entry_count: plan.identity().canonical_basis_entry_count(),
    };
    require_resolution_basis(&basis)?;
    let unresolved = Recipe::<Unresolved, _>::new(plan);
    let resolved = ResolveRecipeTransition
        .transition(
            unresolved,
            RecipeResolutionContext::new(
                basis,
                AuthorityWitness::from_authority_marker(SimulationPlanResolutionAuthority),
            ),
        )
        .into_value();
    let lowered = LowerRecipeTransition::new(CapabilityWitness::from_capability_marker(
        SimulationPlanningCapability,
    ))
    .transition(resolved)
    .into_value();
    let admitted = AdmitRecipeTransition::new(AuthorityWitness::from_authority_marker(
        SimulationPlanAdmissionAuthority,
    ))
    .transition(lowered)
    .into_value();
    Ok(admitted.into_parts().0)
}

fn require_resolution_basis(
    basis: &SimulationPlanResolutionBasis,
) -> Result<(), SimulationPlanDenial> {
    if *basis.digest.bytes() == [0; 32] || basis.canonical_basis_entry_count == 0 {
        return Err(SimulationPlanDenial::ProofProgressionSkipped);
    }
    Ok(())
}

struct SimulationPlanResolutionAuthority;
impl AuthorityMarker for SimulationPlanResolutionAuthority {}

struct SimulationPlanningCapability;
impl CapabilityMarker for SimulationPlanningCapability {}

struct SimulationPlanAdmissionAuthority;
impl AuthorityMarker for SimulationPlanAdmissionAuthority {}
