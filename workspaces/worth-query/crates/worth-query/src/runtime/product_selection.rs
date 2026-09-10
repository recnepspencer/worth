//! Issue the explicit Query token for the installed root product.

use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub fn current_world(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.runtime.installed_product.world.root_product_branch()
    }

    pub fn branches(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranches<'_> {
        self.runtime.installed_product.world.product_branches()
    }
}
