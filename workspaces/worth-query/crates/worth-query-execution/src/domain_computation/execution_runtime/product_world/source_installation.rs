use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchIdentity, RelationalOwnerServicePorts,
};
use worth_runtime_bridge::facade::RuntimeBridgeRelationalSource;

use super::WorthQueryRelationalSourceOwner;

/// Owner-issued inputs for composing the existing graph into a product World.
/// The services, admitted basis, and Bridge registry come from one source owner.
#[doc(hidden)]
pub struct WorthQueryProductRelationalInstallation {
    pub(super) gate: WorthQueryRelationalSourceOwner,
    pub(super) next_ordinal: u64,
    pub(super) services: RelationalOwnerServicePorts,
    pub(super) basis: AdmittedRelationalBranchBasis,
    pub(super) source: RuntimeBridgeRelationalSource,
}

impl WorthQueryRelationalSourceOwner {
    pub fn prepare_product_source(
        &self,
        branch: &RelationalBranchIdentity,
    ) -> Result<WorthQueryProductRelationalInstallation, super::WorthQueryRelationalSourceDenial>
    {
        self.with_runtime(|runtime| {
            let (_, basis) = runtime.observe_branch(branch)?;
            let names = runtime.branch_names();
            let mut highest = 0;
            for name in names.registered().iter().chain(names.retired()) {
                if let Some(ordinal) = crate::basis::product_branch_ordinal(&name.0)
                    .map_err(super::WorthQueryRelationalSourceDenial::InvalidBranchName)?
                {
                    highest = highest.max(ordinal);
                }
            }
            Ok(WorthQueryProductRelationalInstallation {
                gate: self.clone(),
                next_ordinal: highest.saturating_add(1),
                services: runtime.owner_component_services(),
                basis,
                source: self.source.clone(),
            })
        })?
    }
}
