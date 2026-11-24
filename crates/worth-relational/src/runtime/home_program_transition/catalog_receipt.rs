//! The exact receipt selected by the installed Home program owner.
use super::{
    PreparedRelationalHomeProgramContracts, RelationalHomeProgramCatalogReceipt,
    RelationalRootExecutionContracts,
};
use std::sync::Arc;

impl RelationalRootExecutionContracts {
    pub(crate) fn catalog_receipt(self: &Arc<Self>) -> RelationalHomeProgramCatalogReceipt {
        self.identity.clone()
    }
}
impl PreparedRelationalHomeProgramContracts {
    pub fn catalog_receipt(&self) -> RelationalHomeProgramCatalogReceipt {
        self.meaning.catalog_receipt()
    }
}
