//! The runtime binds the exact visibility and record readers once per operation.
use crate::runtime::RelationalRuntime;
use crate::visibility::snapshot_states::{SnapshotReadDirectory, SnapshotRecordReadInputs};
use std::sync::Arc;

impl RelationalRuntime {
    pub(crate) fn snapshot_record_read_inputs(&self) -> SnapshotRecordReadInputs<'_> {
        SnapshotRecordReadInputs::new(
            &self.history,
            &self.services.instrumentation,
            &self.config.schema.registry,
        )
    }
}

impl<'a> From<&'a RelationalRuntime> for SnapshotReadDirectory<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self::new(&runtime.visibility, runtime.runtime_instance_id())
    }
}
impl<'a> From<&'a Arc<RelationalRuntime>> for SnapshotReadDirectory<'a> {
    fn from(runtime: &'a Arc<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
impl<'a> From<&'a mut RelationalRuntime> for SnapshotReadDirectory<'a> {
    fn from(runtime: &'a mut RelationalRuntime) -> Self {
        (&*runtime).into()
    }
}

impl<'a> From<&'a RelationalRuntime>
    for crate::visibility::materialization::read_records::projection::ProjectionInputs<'a>
{
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            history: &runtime.history,
            instrumentation: &runtime.services.instrumentation,
            symbols: &runtime.services.symbols,
            empty_history_schema: &runtime.config.schema.registry,
        }
    }
}
