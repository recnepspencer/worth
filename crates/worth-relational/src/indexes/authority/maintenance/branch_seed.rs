use super::IndexAuthority;
use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId};
impl IndexAuthority<'_> {
    /// The authoring branch's generation for the exact root a fresh fork
    /// selects. Entries are a pure function of the root, so the fork publishes
    /// its own generation from them instead of cold-projecting the graph.
    pub(super) fn fork_seed_admitted(
        &self,
        index_id: DerivedIndexId,
        request: &DerivedIndexBuildRequest,
        authoring: Option<&crate::facade::history::BranchId>,
        version: crate::facade::identity::VersionId,
        schema_version: crate::facade::schema::SchemaVersionId,
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
            .runtime
            .indexes
            .maintenance_inputs_admitted(
                index_id,
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
