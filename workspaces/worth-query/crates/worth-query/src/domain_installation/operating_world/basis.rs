use std::sync::Arc;

use crate::basis_lifecycle::{AdmittedBasisCapability, BasisOperationLane};
use worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease;

/// Lane permission and exact owner custody travel together after product entry.
pub(crate) enum WorthQueryOperatingWorldBasis<L: BasisOperationLane> {
    Component(AdmittedBasisCapability<L>),
    Product {
        capability: AdmittedBasisCapability<L>,
        product: Arc<WorthQueryProductBranchLease>,
    },
}

impl<L: BasisOperationLane> Clone for WorthQueryOperatingWorldBasis<L> {
    fn clone(&self) -> Self {
        match self {
            Self::Component(capability) => Self::Component(capability.clone()),
            Self::Product {
                capability,
                product,
            } => Self::Product {
                capability: capability.clone(),
                product: Arc::clone(product),
            },
        }
    }
}

impl<L: BasisOperationLane> WorthQueryOperatingWorldBasis<L> {
    pub(crate) fn lane(&self) -> &AdmittedBasisCapability<L> {
        match self {
            Self::Component(capability) | Self::Product { capability, .. } => capability,
        }
    }

    pub(crate) fn product(
        &self,
    ) -> Result<
        &Arc<WorthQueryProductBranchLease>,
        super::super::WorthQueryConditionalAdmissionDenial,
    > {
        match self {
            Self::Product { product, .. } => Ok(product),
            Self::Component(_) => {
                Err(super::super::WorthQueryConditionalAdmissionDenial::ProductBasisRequired)
            }
        }
    }
}
