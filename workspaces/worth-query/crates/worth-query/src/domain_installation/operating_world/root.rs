use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryRuntime;

pub struct WorthQueryInstalledOperatingWorld<'runtime, L: BasisOperationLane> {
    pub(super) runtime: &'runtime WorthQueryRuntime,
    pub(super) basis: super::WorthQueryOperatingWorldBasis<L>,
}

impl<'runtime, L: BasisOperationLane> WorthQueryInstalledOperatingWorld<'runtime, L> {
    pub(crate) fn new(
        runtime: &'runtime WorthQueryRuntime,
        basis: super::WorthQueryOperatingWorldBasis<L>,
    ) -> Self {
        Self { runtime, basis }
    }

    /// Inspect the product already retained by this operating world. This does
    /// not resolve the branch again or manufacture a component admission.
    pub fn product_branch(
        &self,
    ) -> Result<
        &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        super::super::WorthQueryConditionalAdmissionDenial,
    > {
        self.basis.product().map(std::sync::Arc::as_ref)
    }

    /// Retain the exact product occurrence already admitted by this operating
    /// world for a longer-lived owner service.
    pub fn retain_product_branch(
        &self,
    ) -> Result<
        std::sync::Arc<worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease>,
        super::super::WorthQueryConditionalAdmissionDenial,
    > {
        self.basis.product().cloned()
    }

    pub fn family<F>(
        &self,
        _family: F,
    ) -> super::WorthQueryOperationFamilyView<'_, 'runtime, F, L> {
        super::WorthQueryOperationFamilyView::new(self)
    }
}
