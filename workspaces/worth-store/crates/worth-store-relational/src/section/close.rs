use super::{PreparedRelationalSection, RelationalSectionCheckpointDenial, RelationalStoreSection};
use std::sync::Arc;
use worth_relational::facade::runtime::{
    PreparedRelationalRuntimeClose, RelationalRuntimeCloseDenial,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub struct ClosingRelationalSection {
    pub(super) source: Option<super::readmission::source::ReadmittedRelationalSource>,
    runtime: PreparedRelationalRuntimeClose,
    cold_source: Arc<crate::cold_source::StoreColdSource>,
    serving: Arc<ServingPhysicalRuntime>,
    checkpoint_selection: Arc<dyn super::RelationalCheckpointSelectionSource>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalSectionCloseDenial {
    SharedRuntimeCustody,
    Native(RelationalRuntimeCloseDenial),
}

impl RelationalStoreSection {
    pub fn prepare_close(
        self,
    ) -> Result<ClosingRelationalSection, (Self, RelationalSectionCloseDenial)> {
        let Self {
            source,
            runtime,
            cold_source,
            serving,
            checkpoint_selection,
        } = self;
        let runtime = match Arc::try_unwrap(runtime) {
            Ok(runtime) => runtime,
            Err(runtime) => {
                return Err((
                    Self {
                        source,
                        runtime,
                        cold_source,
                        serving,
                        checkpoint_selection,
                    },
                    RelationalSectionCloseDenial::SharedRuntimeCustody,
                ))
            }
        };
        let attached = super::unshared::AttachedRelationalRuntime {
            runtime,
            cold_source,
            serving,
            checkpoint_selection,
        };
        match prepare_attached_close(attached) {
            Ok(mut closing) => {
                closing.source = source;
                Ok(closing)
            }
            Err((attached, cause)) => Err((attached.share(source), cause)),
        }
    }
}

pub(super) fn prepare_attached_close(
    attached: super::unshared::AttachedRelationalRuntime,
) -> Result<
    ClosingRelationalSection,
    (
        super::unshared::AttachedRelationalRuntime,
        RelationalSectionCloseDenial,
    ),
> {
    let super::unshared::AttachedRelationalRuntime {
        runtime,
        cold_source,
        serving,
        checkpoint_selection,
    } = attached;
    match runtime.prepare_close() {
        Ok(runtime) => Ok(ClosingRelationalSection {
            runtime,
            cold_source,
            serving,
            checkpoint_selection,
            source: None,
        }),
        Err(refused) => {
            let cause = RelationalSectionCloseDenial::Native(refused.reason());
            Err((
                super::unshared::AttachedRelationalRuntime {
                    runtime: refused.into_runtime(),
                    cold_source,
                    serving,
                    checkpoint_selection,
                },
                cause,
            ))
        }
    }
}

impl ClosingRelationalSection {
    pub fn prepare_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRelationalSection, RelationalSectionCheckpointDenial> {
        let members = self.cold_source.member_locators();
        let checkpoint = self.runtime.native_checkpoint(execution,
            worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &members, source: Some(self.cold_source.clone()),
            },
        ).map_err(RelationalSectionCheckpointDenial::Capture)?;
        super::checkpoint::prepare_captured(&self.cold_source, checkpoint, execution)
            .map(|prepared| prepared.with_selection(self.checkpoint_selection.clone()))
    }

    pub fn resume(self) -> RelationalStoreSection {
        RelationalStoreSection {
            source: self.source,
            runtime: Arc::new(self.runtime.resume()),
            cold_source: self.cold_source,
            serving: self.serving,
            checkpoint_selection: self.checkpoint_selection,
        }
    }

    pub fn finish(self) {
        self.runtime.finish();
        drop(self.cold_source);
        drop(self.serving);
    }
}
