use worth_runtime_world::facade::{
    ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial, RuntimeWorldServiceDenial,
};

use super::WorthQueryProductRuntime;
use crate::basis::WorthQueryProductBranchLease;

impl WorthQueryProductRuntime {
    /// Returns all component retirement work described by World. Query removes
    /// only coordination for the exact successfully retired occurrence.
    pub(crate) fn retire_product_branch(
        &self,
        observed: &WorthQueryProductBranchLease,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    > {
        let report = self
            .fields()
            .owner
            .branch_port()
            .retire_product_branch(observed.observation())?;
        self.fields().activations.release(
            observed.branch_identity(),
            observed.observation().lifecycle_incarnation(),
        );
        Ok(report)
    }

    pub(crate) fn retire_product_branch_with_request(
        &self,
        observed: &WorthQueryProductBranchLease,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        ProductBranchRetirementReport,
        RuntimeWorldServiceDenial<RuntimeWorldBranchRetirementDenial>,
    > {
        let report = self
            .fields()
            .owner
            .branch_port()
            .retire_product_branch_with_request(observed.observation(), execution)?;
        self.fields().activations.release(
            observed.branch_identity(),
            observed.observation().lifecycle_incarnation(),
        );
        Ok(report)
    }
}

mod court_doors_for_test;
