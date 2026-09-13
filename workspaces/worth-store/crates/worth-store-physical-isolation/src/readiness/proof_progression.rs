#[cfg(feature = "certification-authority")]
use worth_proof::{
    AdmitRecipeTransition, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    ContextualTransition, LowerRecipeTransition, RecipeResolutionContext, ResolveRecipeTransition,
    Transition,
};
use worth_proof::{
    AssumptionBasis, CurrentValidity, FreshnessScopedBasis, Lowered, Recipe, Resolved, Unresolved,
};

use super::{PhysicalIsolationEntryIdentity, PhysicalIsolationRootEpochBasis};

pub type PhysicalIsolationResolvedEntryRecipe = Recipe<
    Resolved,
    PhysicalIsolationEntryProofRequest,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<RecoveryReadinessBasis>>,
>;
pub type PhysicalIsolationLoweredEntryRecipe = Recipe<
    Lowered,
    PhysicalIsolationEntryProofRequest,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<RecoveryReadinessBasis>>,
>;
pub type PhysicalIsolationAdmittedEntryRecipe = Recipe<
    worth_proof::Admitted,
    PhysicalIsolationEntryProofRequest,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<RecoveryReadinessBasis>>,
>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalIsolationEntryProofRequest {
    identity: PhysicalIsolationEntryIdentity,
}

#[derive(Debug, Clone)]
pub struct RecoveryReadinessBasis {
    identity: PhysicalIsolationEntryIdentity,
    root_epoch_basis: PhysicalIsolationRootEpochBasis,
}

#[derive(Debug, Clone)]
pub struct PhysicalIsolationEntryProofProgression {
    unresolved: Recipe<Unresolved, PhysicalIsolationEntryProofRequest>,
    resolved: PhysicalIsolationResolvedEntryRecipe,
    lowered: PhysicalIsolationLoweredEntryRecipe,
    admitted: PhysicalIsolationAdmittedEntryRecipe,
}

#[cfg(feature = "certification-authority")]
pub struct S5EntryAuthority {
    _private: (),
}

#[cfg(feature = "certification-authority")]
impl AuthorityMarker for S5EntryAuthority {}

#[cfg(feature = "certification-authority")]
struct S5EntryLoweringCapability {
    _private: (),
}

#[cfg(feature = "certification-authority")]
impl CapabilityMarker for S5EntryLoweringCapability {}

impl PhysicalIsolationEntryProofProgression {
    #[cfg(feature = "certification-authority")]
    pub(crate) fn from_identity(identity: PhysicalIsolationEntryIdentity) -> Self {
        let unresolved = Recipe::<Unresolved, _>::new(PhysicalIsolationEntryProofRequest {
            identity: identity.clone(),
        });
        let resolved = ResolveRecipeTransition
            .transition(
                unresolved.clone(),
                RecipeResolutionContext::new(
                    RecoveryReadinessBasis {
                        identity: identity.clone(),
                        root_epoch_basis: identity.root_epoch_basis(),
                    },
                    physical_isolation_entry_authority_witness(),
                ),
            )
            .into_value();
        let lowered = LowerRecipeTransition::new(physical_isolation_entry_lowering_capability())
            .transition(resolved.clone())
            .into_value();
        let admitted = AdmitRecipeTransition::new(physical_isolation_entry_authority_witness())
            .transition(lowered.clone())
            .into_value();
        Self {
            unresolved,
            resolved,
            lowered,
            admitted,
        }
    }

    pub const fn unresolved_recipe(
        &self,
    ) -> &Recipe<Unresolved, PhysicalIsolationEntryProofRequest> {
        &self.unresolved
    }

    pub const fn resolved_recipe(&self) -> &PhysicalIsolationResolvedEntryRecipe {
        &self.resolved
    }

    pub const fn lowered_recipe(&self) -> &PhysicalIsolationLoweredEntryRecipe {
        &self.lowered
    }

    pub const fn admitted_recipe(&self) -> &PhysicalIsolationAdmittedEntryRecipe {
        &self.admitted
    }

    pub const fn is_store_physical_stability_authority(&self) -> bool {
        false
    }
}

impl PhysicalIsolationEntryProofRequest {
    pub const fn identity(&self) -> &PhysicalIsolationEntryIdentity {
        &self.identity
    }
}

impl RecoveryReadinessBasis {
    pub const fn identity(&self) -> &PhysicalIsolationEntryIdentity {
        &self.identity
    }

    pub const fn root_epoch_basis(&self) -> PhysicalIsolationRootEpochBasis {
        self.root_epoch_basis
    }
}

#[cfg(feature = "certification-authority")]
fn physical_isolation_entry_authority_witness() -> AuthorityWitness<S5EntryAuthority> {
    AuthorityWitness::from_authority_marker(S5EntryAuthority { _private: () })
}

#[cfg(feature = "certification-authority")]
fn physical_isolation_entry_lowering_capability() -> CapabilityWitness<S5EntryLoweringCapability> {
    CapabilityWitness::from_capability_marker(S5EntryLoweringCapability { _private: () })
}
