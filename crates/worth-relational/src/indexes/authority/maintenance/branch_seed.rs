use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId};
impl super::owner::IndexMaintenanceOwner<'_> {
    /// The authoring branch's generation for the exact root a fresh fork
    /// selects. Entries are a pure function of the root, so the fork publishes
    /// its own generation from them instead of cold-projecting the graph.
    pub(super) fn fork_seed_admitted(
        &self,
        index_id: DerivedIndexId,
        request: &DerivedIndexBuildRequest,
        authoring: Option<&crate::history::data::coordinates::BranchId>,
        version: crate::identity::data::VersionId,
        schema_version: crate::schema::data::SchemaVersionId,
        work: &mut super::work::MaintenanceWork,
    ) -> Result<
        Option<std::sync::Arc<DerivedIndexGeneration>>,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind,
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
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    crate::indexes::data::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
                }
            })?;
        Ok(selected
            .and_then(|(_, generation, _)| generation)
            .filter(|generation| generation.applicability.schema_version == schema_version))
    }
}
