use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexGeneration};
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotHandle;

pub(super) fn exact_published_generation(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    definition: &DerivedIndexDefinition,
) -> Option<std::sync::Arc<DerivedIndexGeneration>> {
    let branch_id = snapshot.branch_id();
    let schema_version = runtime
        .read_truth()
        .query_plan_context(snapshot)?
        .schema_version;
    runtime.indexes.exact_generation(
        definition.index_id,
        definition.branch_scoped.then_some(branch_id),
        snapshot.version_id,
        schema_version,
    )
}
