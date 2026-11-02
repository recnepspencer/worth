use super::{
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityIssuer, RuntimeWorldOwnerIdentity,
};
use crate::durability::{WorldDurableHomeBinding, WorldIdentityFrontiers};
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
            next_publication_attempt: frontiers.attempt,
            next_product_unpublished: frontiers.custody,
        })
    }
}
