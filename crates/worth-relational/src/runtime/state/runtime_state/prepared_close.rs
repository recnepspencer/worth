//! Bounded owner close using the existing admission and publication registries.
use super::RelationalRuntime;
pub use crate::change_source::RelationalReadCustodyHolders;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalRuntimeCloseDenial {
    NotOwner,
    OperationsInFlight,
    PublicationCustodyRetained,
    ReadCustodyRetained(RelationalReadCustodyHolders),
}

/// A refused close still owns the same serving runtime.
#[derive(Debug)]
pub struct RelationalRuntimeCloseRefused {
    runtime: Box<RelationalRuntime>,
    reason: RelationalRuntimeCloseDenial,
}

impl RelationalRuntimeCloseRefused {
    pub fn reason(&self) -> RelationalRuntimeCloseDenial {
        self.reason
    }
    pub fn into_runtime(self) -> RelationalRuntime {
        *self.runtime
    }
}

/// Admission is stopped, and the actual owner registries contain no admitted
/// work, publication settlement or read custody. Only owner maintenance may
/// run before finish; no new service operation can enter.
///
/// ```compile_fail,E0599
/// use worth_relational::facade::runtime::PreparedRelationalRuntimeClose;
/// fn reopen_mutation_doors(closing: PreparedRelationalRuntimeClose) { closing.runtime(); }
/// ```
#[derive(Debug)]
pub struct PreparedRelationalRuntimeClose {
    runtime: RelationalRuntime,
}

impl PreparedRelationalRuntimeClose {
    /// The stopped phase exposes only native maintenance, never the ordinary
    /// runtime whose mutation doors require a serving owner.
    pub fn native_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        placement: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        self.runtime
            .durability_authority()
            .native_checkpoint_with_policy(execution, execution.allocation_policy(), placement)
    }

    /// Restore the same serving owner after a composition-level refusal.
    pub fn resume(self) -> RelationalRuntime {
        if let Some(close) = self.runtime.tenure.close_authority() {
            close.resume_admission();
        }
        self.runtime
    }

    pub fn finish(self) {
        drop(self.runtime);
    }
}

impl RelationalRuntime {
    /// Stop admission without waiting beyond the supplied caller's opportunity
    /// to retry. This consults the real in-flight and custody registries; shared
    /// allocation reference counts are not completion evidence.
    pub fn prepare_close(
        self,
    ) -> Result<PreparedRelationalRuntimeClose, RelationalRuntimeCloseRefused> {
        let reason = match self.tenure.close_authority() {
            None => Some(RelationalRuntimeCloseDenial::NotOwner),
            Some(close) => {
                if !close.stop_if_quiescent() {
                    Some(RelationalRuntimeCloseDenial::OperationsInFlight)
                } else if self.state.publication_owner.has_live_publication_custody() {
                    close.resume_admission();
                    Some(RelationalRuntimeCloseDenial::PublicationCustodyRetained)
                } else if self.state.visibility.active_snapshot_count() != 0
                    || self.state.history.has_detached_retention_obligations()
                {
                    close.resume_admission();
                    Some(RelationalRuntimeCloseDenial::ReadCustodyRetained(
                        RelationalReadCustodyHolders::SnapshotsAndHistory {
                            active_snapshots: self.state.visibility.active_snapshot_count(),
                            detached_history: self
                                .state
                                .history
                                .has_detached_retention_obligations(),
                        },
                    ))
                } else {
                    None
                }
            }
        };
        match reason {
            Some(reason) => Err(RelationalRuntimeCloseRefused {
                runtime: Box::new(self),
                reason,
            }),
            None => Ok(PreparedRelationalRuntimeClose { runtime: self }),
        }
    }
}
