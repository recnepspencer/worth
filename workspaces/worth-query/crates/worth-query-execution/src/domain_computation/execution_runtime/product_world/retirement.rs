use worth_runtime_world::facade::ProductBranchRetirementReport;

use super::WorthQueryProductRuntime;
use crate::basis::WorthQueryProductBranchLease;

impl WorthQueryProductRuntime {
    /// Returns all component retirement work described by World. Query removes
    /// only coordination for the exact successfully retired occurrence.
    pub(crate) fn retire_product_branch(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<ProductBranchRetirementReport, super::WorthQueryProductBranchCloseDenial> {
        self.gate
            .with_runtime(|_| {
                let report = self
                    .owner
                    .branch_port()
                    .retire_product_branch(observed.observation())
                    .map_err(super::branch_close::map_retirement_denial)?;
                self.activations.release(
                    observed.branch_identity(),
                    observed.observation().lifecycle_incarnation(),
                );
                Ok(report)
            })
            .map_err(super::WorthQueryProductBranchCloseDenial::Handle)?
    }
}
