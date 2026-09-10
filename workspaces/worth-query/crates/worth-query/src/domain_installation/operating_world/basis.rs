use std::sync::Arc;

use crate::basis_lifecycle::{AdmittedBasisCapability, BasisOperationLane};
use worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease;

/// Lane permission and exact owner custody travel together after product entry.
pub(crate) struct WorthQueryOperatingWorldBasis<L: BasisOperationLane> {
    capability: AdmittedBasisCapability<L>,
    product: Arc<WorthQueryProductBranchLease>,
}

impl<L: BasisOperationLane> Clone for WorthQueryOperatingWorldBasis<L> {
    fn clone(&self) -> Self {
        Self {
            capability: self.capability.clone(),
            product: Arc::clone(&self.product),
        }
    }
}

impl<L: BasisOperationLane> WorthQueryOperatingWorldBasis<L> {
    pub(crate) fn new(
        capability: AdmittedBasisCapability<L>,
        product: WorthQueryProductBranchLease,
    ) -> Self {
        Self {
            capability,
            product: Arc::new(product),
        }
    }

    pub(crate) fn lane(&self) -> &AdmittedBasisCapability<L> {
        &self.capability
    }

    pub(crate) fn product(&self) -> &Arc<WorthQueryProductBranchLease> {
        &self.product
    }
}
