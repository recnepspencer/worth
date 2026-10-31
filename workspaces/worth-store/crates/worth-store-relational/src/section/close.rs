use super::{PreparedRelationalSection, RelationalSectionCheckpointDenial, RelationalStoreSection};
use std::sync::Arc;
use worth_relational::facade::runtime::{
    PreparedRelationalRuntimeClose, RelationalRuntimeCloseDenial,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub struct ClosingRelationalSection {
    runtime: PreparedRelationalRuntimeClose,
    cold_source: Arc<crate::cold_source::StoreColdSource>,
    serving: Arc<ServingPhysicalRuntime>,
}

impl RelationalStoreSection {
    pub fn prepare_close(
        self,
    ) -> Result<ClosingRelationalSection, (Self, RelationalRuntimeCloseDenial)> {
        let Self {
            runtime,
            cold_source,
            serving,
        } = self;
        match runtime.prepare_close() {
            Ok(runtime) => Ok(ClosingRelationalSection {
                runtime,
                cold_source,
                serving,
            }),
            Err(refused) => {
                let reason = refused.reason();
                Err((
                    Self {
                        runtime: refused.into_runtime(),
                        cold_source,
                        serving,
                    },
                    reason,
                ))
            }
        }
    }
}

impl ClosingRelationalSection {
    pub fn prepare_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalSection<'_>, RelationalSectionCheckpointDenial> {
        let members = self.cold_source.member_locators();
        let checkpoint = self.runtime.native_checkpoint(execution,
            worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members, source: Some(self.cold_source.clone()),
            },
        ).map_err(RelationalSectionCheckpointDenial::Capture)?;
        super::checkpoint::prepare_captured(&self.cold_source, checkpoint, execution)
    }

    pub fn resume(self) -> RelationalStoreSection {
        RelationalStoreSection {
            runtime: self.runtime.resume(),
            cold_source: self.cold_source,
            serving: self.serving,
        }
    }

    pub fn finish(self) {
        self.runtime.finish();
        drop(self.cold_source);
        drop(self.serving);
    }
}
