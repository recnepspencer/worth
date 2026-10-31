use super::{PreparedRelationalSection, RelationalSectionCheckpointDenial, RelationalStoreSection};
use std::sync::Arc;
use worth_relational::facade::runtime::{
    PreparedRelationalRuntimeClose, RelationalRuntimeCloseDenial,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;

pub struct ClosingRelationalSection {
    recovered: Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    recovery_outcome: Option<worth_relational::facade::durability::RecoveryOutcome>,
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
            recovered,
            recovery_outcome,
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
                        recovered,
                        recovery_outcome,
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
        match prepare_attached_close(attached, recovered, recovery_outcome) {
            Ok(closing) => Ok(closing),
            Err((attached, recovered, outcome, cause)) => {
                Err((attached.share(recovered, outcome), cause))
            }
        }
    }
}

pub(super) fn prepare_attached_close(
    attached: super::unshared::AttachedRelationalRuntime,
    recovered: Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    recovery_outcome: Option<worth_relational::facade::durability::RecoveryOutcome>,
) -> Result<
    ClosingRelationalSection,
    (
        super::unshared::AttachedRelationalRuntime,
        Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
        Option<worth_relational::facade::durability::RecoveryOutcome>,
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
            recovered,
            recovery_outcome,
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
                recovered,
                recovery_outcome,
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
            recovered: self.recovered,
            recovery_outcome: self.recovery_outcome,
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
