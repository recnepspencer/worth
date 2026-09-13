mod attempt;
mod basis_key;
mod bootstrap;
mod branch;
mod commit;
mod owner;
mod retained_partial;

pub use attempt::CompositePublicationAttemptIdentity;
pub use basis_key::CompositeBasisKey;
pub use bootstrap::RuntimeWorldBootstrapAttemptIdentity;
pub use branch::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchReferenceGeneration,
};
pub use commit::CompositeCommitIdentity;
pub(crate) use owner::RuntimeWorldIdentityIssuer;
pub use owner::{
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityFamily, RuntimeWorldOwnerIdentity,
};
pub use retained_partial::ProductUnpublishedOwnerEffectsIdentity;

pub(crate) fn issuer_for_owner_construction(
    capability: &crate::lifecycle::owner::RuntimeWorldOwnerConstructionCapability,
) -> Result<(RuntimeWorldIdentityIssuer, RuntimeWorldOwnerIdentity), RuntimeWorldIdentityExhaustion>
{
    owner::RuntimeWorldIdentityIssuer::from_owner_construction(capability)
}
