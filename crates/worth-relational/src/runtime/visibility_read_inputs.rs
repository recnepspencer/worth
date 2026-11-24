//! The runtime binds the exact visibility and record readers once per operation.
use crate::runtime::RelationalRuntime;
use crate::visibility::snapshot_states::{
    resolution::{SnapshotReadDirectory, SnapshotResolutionInputs},
    SnapshotRecordReadInputs,
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
            history: runtime.history.canonical_read(),
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

use crate::branch::basis::retirement::BranchBasisRetirementOwner;

impl From<&RelationalRuntime> for BranchBasisRetirementOwner {
    fn from(runtime: &RelationalRuntime) -> Self {
        Self::installed(runtime.runtime_instance_id())
    }
}

use crate::branch::basis_observation_inputs::BranchObservationInputs;
use crate::branch::head_visibility_inputs::BranchHeadVisibilityInputs;
use crate::branch::reference::world_selection::WorldSourceSelection;

impl<'a> From<&'a RelationalRuntime> for BranchObservationInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            history: &runtime.history,
            runtime_instance_id: runtime.runtime_instance_id(),
            empty_schema: &runtime.config.schema.registry,
            instrumentation: &runtime.services.instrumentation,
        }
    }
}

impl<'a> From<&'a RelationalRuntime> for BranchHeadVisibilityInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            observations: runtime.into(),
            directory: &runtime.visibility,
            cache: runtime.into(),
        }
    }
}

impl<'a> From<&'a RelationalRuntime> for WorldSourceSelection<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            history: &runtime.history,
            visibility: runtime.into(),
        }
    }
}

use crate::visibility::snapshot_observation::SnapshotObservationInputs;

impl<'a> From<&'a RelationalRuntime> for SnapshotObservationInputs<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            symbols: &runtime.services.symbols,
            visibility: &runtime.visibility,
            history: &runtime.history,
            maximum_active_snapshots: runtime
                .config
                .publication
                .policy
                .max_active_snapshot_handles,
            cache: runtime.into(),
        }
    }
}

impl<'a> From<&'a Arc<RelationalRuntime>> for SnapshotObservationInputs<'a> {
    fn from(runtime: &'a Arc<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
