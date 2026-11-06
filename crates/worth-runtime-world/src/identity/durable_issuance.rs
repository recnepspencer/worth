use super::{
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityIssuer, RuntimeWorldOwnerIdentity,
};
use crate::durability::{WorldDurableHomeBinding, WorldIdentityFrontiers};
#[cfg(test)]
mod tests;
impl RuntimeWorldIdentityIssuer {
    pub(crate) fn from_home_construction(
        capability: &crate::lifecycle::owner::RuntimeWorldOwnerConstructionCapability,
        binding: &WorldDurableHomeBinding,
        frontiers: WorldIdentityFrontiers,
    ) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        let _ = capability;
        Ok(Self {
            owner: RuntimeWorldOwnerIdentity::for_admitted_home(
                binding.owner(),
                binding.incarnation(),
            )?,
            next_branch_incarnation: frontiers.branch,
            next_composite_commit: frontiers.commit,
            next_bootstrap_attempt: frontiers.bootstrap,
            next_publication_attempt: frontiers
                .attempt
                .max(binding.transition_attempt_frontier().unwrap_or(0)),
            next_product_unpublished: frontiers.custody,
        })
    }
}
