//! The index owner required by native preparation: its actual optional Home catalog.
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionCatalog, PreparedRelationalIndexDefinitions,
};
use std::sync::Arc;
pub struct IndexMaintenanceOwner<'a> {
    pub indexes: &'a crate::runtime::state::subsystems::indexing::IndexingSubsystem,
    pub definitions: Option<&'a PreparedRelationalIndexDefinitions>,
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
        root: &worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    ) -> Result<
        Arc<worth_relational_contracts::internal_port::indexes::data::installed_routing::InstalledIndexRouting>,
        super::DerivedIndexMaintenanceDenialKind,
    >{
        self.catalog()
            .routing(root.schema_authority().authority_digest())
            .ok_or(super::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)
    }
}
