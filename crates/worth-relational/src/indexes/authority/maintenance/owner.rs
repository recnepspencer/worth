//! The index owner required by native preparation: its actual optional Home catalog.
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionCatalog, PreparedRelationalIndexDefinitions,
};
use std::sync::Arc;
pub(super) struct IndexMaintenanceOwner<'a> {
    pub(super) indexes: &'a crate::runtime::state::subsystems::indexing::IndexingSubsystem,
    pub(super) definitions: Option<&'a PreparedRelationalIndexDefinitions>,
}
impl IndexMaintenanceOwner<'_> {
    pub(super) fn prepared_catalog(&self) -> Option<&IndexDefinitionCatalog> {
        self.definitions
            .map(|prepared| prepared.captured.catalog.as_ref())
    }
    pub(super) fn catalog(&self) -> Arc<IndexDefinitionCatalog> {
        self.definitions
            .map(|prepared| prepared.captured.catalog.clone())
            .unwrap_or_else(|| self.indexes.definition_read_binding().snapshot())
    }
    pub(super) fn routes(
        &self,
        root: &crate::branch::root::RelationalBranchRoot,
    ) -> Result<
        Arc<crate::indexes::data::InstalledIndexRouting>,
        super::DerivedIndexMaintenanceDenialKind,
    > {
        self.catalog()
            .routing(root.schema_authority().authority_digest())
            .ok_or(super::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)
    }
}
