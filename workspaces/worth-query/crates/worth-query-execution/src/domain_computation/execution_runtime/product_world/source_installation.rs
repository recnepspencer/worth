use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchIdentity,
    RelationalOwnerServicePorts,
};
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;

use super::WorthQueryRelationalSourceOwner;

/// Owner-issued inputs for composing the existing graph into a product World.
/// The services, admitted basis, and Bridge registry come from one source owner.
#[doc(hidden)]
pub struct WorthQueryProductRelationalInstallation {
    pub(super) services: RelationalOwnerServicePorts,
    pub(super) basis: AdmittedRelationalBranchBasis,
    pub(super) source: RuntimeBridgeRelationalSource,
}

impl WorthQueryRelationalSourceOwner {
    pub fn prepare_product_source(
        &self,
        branch: &RelationalBranchIdentity,
    ) -> Result<WorthQueryProductRelationalInstallation, RelationalBranchBasisDenial> {
        self.with_runtime(|runtime| {
            let (_, basis) = runtime.observe_branch(branch)?;
            Ok(WorthQueryProductRelationalInstallation {
                services: runtime.owner_component_services(),
                basis,
                source: self.source.clone(),
            })
        })
    }
}
