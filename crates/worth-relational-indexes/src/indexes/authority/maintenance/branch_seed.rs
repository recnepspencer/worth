use crate::contracts::indexes::data::definition::DerivedIndexId;
use worth_relational_branch::internal_port::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexGeneration,
};
impl super::owner::IndexMaintenanceOwner<'_> {
    /// The authoring branch's generation for the exact root a fresh fork
    /// selects. Entries are a pure function of the root, so the fork publishes
    /// its own generation from them instead of cold-projecting the graph.
    pub(super) fn fork_seed_admitted(
        &self,
        index_id: DerivedIndexId,
        request: &DerivedIndexBuildRequest,
        authoring: Option<&crate::contracts::history::data::coordinates::BranchId>,
        version: crate::contracts::identity::data::primitives::VersionId,
        schema_version: crate::contracts::schema::data::SchemaVersionId,
        work: &mut super::work::MaintenanceWork,
    ) -> Result<
        Option<std::sync::Arc<DerivedIndexGeneration>>,
        crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind,
    > {
        let Some(authoring) = authoring else {
            return Ok(None);
        };
        if authoring == &request.branch_id {
            return Ok(None);
        }
        let selected = self
            .indexes
            .maintenance_inputs_admitted(
                index_id,
                self.prepared_catalog(),
                authoring,
                request.source_commit_id,
                version,
                None,
                |units, bytes| work.prepare(units, bytes),
            )
            .map_err(|stop| match stop {
                crate::contracts::indexes::data::maintenance::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                crate::contracts::indexes::data::maintenance::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
                }
            })?;
        Ok(selected
            .and_then(|(_, generation, _)| generation)
            .filter(|generation| generation.applicability.schema_version == schema_version))
    }
}
