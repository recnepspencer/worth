//! The runtime binds the exact visibility and record readers once per operation.
use crate::runtime::RelationalRuntime;
use crate::visibility::snapshot_states::{
    SnapshotReadDirectory, SnapshotRecordReadInputs, SnapshotResolutionInputs,
};
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

use crate::visibility::branch_scope::HistoricalBasisInputs;
use crate::visibility::cache_state::VisibilityCacheInputs;

impl<'a> From<&'a RelationalRuntime> for HistoricalBasisInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            history: &runtime.history,
            genesis_branch: &runtime.config.history.main_branch,
        }
    }
}

impl<'a> From<&'a RelationalRuntime> for VisibilityCacheInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            cache: &runtime.visibility.cache,
            policy: &runtime.config.visibility.cache_policy,
            history: runtime.into(),
            symbols: &runtime.services.symbols,
            records: runtime.snapshot_record_read_inputs(),
            instrumentation: &runtime.services.instrumentation,
        }
    }
}

impl<'a> From<&'a RelationalRuntime> for SnapshotResolutionInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            directory: runtime.into(),
            cache: runtime.into(),
        }
    }
}
