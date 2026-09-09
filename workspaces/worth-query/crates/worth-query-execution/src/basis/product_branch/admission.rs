use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn admit_current_product_branch(
        &self,
    ) -> Result<WorthQueryProductBranchLease, WorthQueryProductBranchAdmissionDenial> {
        self.product_runtime
            .admit_product_branch(self.product_runtime.default_branch())
    }

    pub(crate) fn admit_product_publication(
        &self,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        WorthQueryProductBranchAdmissionDenial,
    >{
        self.product_runtime.admit_product_publication()
    }
}
