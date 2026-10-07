use super::IndexAuthority;
use crate::indexes::data::{DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId};
impl IndexAuthority<'_> {
    /// The authoring branch's generation for the exact root a fresh fork
    /// selects. Entries are a pure function of the root, so the fork publishes
    /// its own generation from them instead of cold-projecting the graph.
    pub(super) fn fork_seed(
        &self,
        index_id: DerivedIndexId,
        request: &DerivedIndexBuildRequest,
        authoring: Option<&crate::facade::history::BranchId>,
        version: crate::facade::identity::VersionId,
        schema_version: crate::facade::schema::SchemaVersionId,
    ) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
        let authoring = authoring?;
        if authoring == &request.branch_id {
            return None;
        }
        self.runtime
            .indexes
            .published_generation_for_commit(
                index_id,
                Some(authoring),
                request.source_commit_id,
                version,
            )
            .filter(|generation| generation.applicability.schema_version == schema_version)
    }
}
