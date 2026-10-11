//! Generation routing queries used by the test-only index execution entry.
use super::{
    Arc, BranchId, DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexId,
    IndexingSubsystem, SchemaVersionId, VersionId,
};

impl IndexingSubsystem {
    pub(crate) fn generation(
        &self,
        id: DerivedIndexGenerationId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.generation(id)
    }

    pub(crate) fn candidate_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state
            .read()
            .generations
            .candidate(index, branch, version, schema)
    }

    pub(crate) fn any_generation_at_or_before(&self, version: VersionId) -> bool {
        self.state.read().generations.any_at_or_before(version)
    }
}
