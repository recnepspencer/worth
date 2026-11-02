//! A fresh association header shares the exact fork source's immutable truth.
use super::*;

impl RelationalBranchRoot {
    pub(crate) fn prepare_managed_fork_header(
        &self,
        issuer: &RelationalBranchRootIdentityIssuer,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let id = issuer.issue_root_id()?;
        let mut header = self.clone();
        header.id = id;
        header.managed_source_image = std::sync::OnceLock::new();
        Ok(Arc::new(header))
    }

    pub(crate) fn shares_authoritative_regions_with(&self, source: &Self) -> bool {
        Arc::ptr_eq(&self.regions, &source.regions)
            && Arc::ptr_eq(&self.schema_authority, &source.schema_authority)
    }
}
