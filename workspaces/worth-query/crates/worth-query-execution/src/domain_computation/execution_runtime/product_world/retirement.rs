use worth_runtime_world::facade::{
    ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial, RuntimeWorldServiceDenial,
};

use super::WorthQueryProductRuntime;
use crate::basis::WorthQueryProductBranchLease;

impl WorthQueryProductRuntime {
    /// Returns all component retirement work described by World. Query removes
    /// only coordination for the exact successfully retired occurrence.
    pub fn retire_product_branch(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    > {
        let report = self
            .owner
            .branch_port()
            .retire_product_branch(observed.observation())?;
        self.activations.release(
            observed.branch_identity(),
            observed.observation().lifecycle_incarnation(),
        );
        Ok(report)
    }
}
