//! Versioned comparison data; operational custody is always freshly admitted.
use serde::{Deserialize, Serialize};
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;
use worth_runtime_world::facade::{ProductBranchObservation, ProductBranchReferenceDescription};

const EXPECTED_PRODUCT_BASIS_DESCRIPTION_VERSION: u16 = 1;

/// Store this value as ordinary domain data, then ask the installed runtime to
/// select it again. Its fields retain neither readers nor execution authority.
/// Coordinates are scoped to the original live installation, not process-restart
/// restoration or a portable model identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorthQueryExpectedProductBasisDescription {
    format_version: u16,
    #[serde(with = "world_encoding::ReferenceDescription")]
    world: ProductBranchReferenceDescription,
    native: RelationalBranchBasisDescriptor,
}

impl WorthQueryExpectedProductBasisDescription {
    pub(crate) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            format_version: EXPECTED_PRODUCT_BASIS_DESCRIPTION_VERSION,
            world: ProductBranchReferenceDescription::from_observation(observation),
            native: observation.basis().relational_basis().descriptor().clone(),
        }
    }

    pub(crate) fn version(&self) -> (u16, u16) {
        (
            EXPECTED_PRODUCT_BASIS_DESCRIPTION_VERSION,
            self.format_version,
        )
    }

    pub(crate) fn world(&self) -> &ProductBranchReferenceDescription {
        &self.world
    }

    pub(crate) fn native(&self) -> &RelationalBranchBasisDescriptor {
        &self.native
    }
}

mod world_encoding;
