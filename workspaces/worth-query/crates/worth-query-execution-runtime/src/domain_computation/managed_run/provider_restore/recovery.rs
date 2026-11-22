use std::sync::Arc;

use super::super::retained_graph_execution::WorthQueryRetainedManagedGraphExecution;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderMemoryArena;
use crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionReleaseEvidence;
use crate::domain_computation::{
    WorthQueryProviderCheckpointEvidence, WorthQueryProviderCheckpointReleaseEvidence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedGraphRestoreRecoveryKind {
    ProviderRestorePanicked,
    ProviderRestoreRejectedAfterExecutionAdmission,
    RestoredExecutionReleaseRecoveryRequired,
    CheckpointReleasePanicked,
}

pub struct WorthQueryManagedGraphRestoreRecoveryRequired {
    kind: WorthQueryManagedGraphRestoreRecoveryKind,
    detail: Arc<str>,
    resource: WorthQueryManagedGraphRestoreRecoveryResource,
}

pub struct WorthQueryManagedGraphRestoreCleanupRequired {
    checkpoint: WorthQueryManagedGraphRestoreCleanupCheckpoint,
    restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
}

pub struct WorthQueryManagedGraphRestoreCleanupEvidence {
    pub memory: WorthQueryGraphProviderMemoryArena,
    pub checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    pub restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
}

enum WorthQueryManagedGraphRestoreRecoveryResource {
    Retained(WorthQueryRetainedManagedGraphExecution),
    RetainedAfterRestoredRelease {
        retained: WorthQueryRetainedManagedGraphExecution,
        restored_execution: WorthQueryProviderExecutionReleaseEvidence,
    },
    Released {
        memory: WorthQueryGraphProviderMemoryArena,
        checkpoint: WorthQueryProviderCheckpointReleaseEvidence,
        restored_execution: WorthQueryProviderExecutionReleaseEvidence,
    },
}

enum WorthQueryManagedGraphRestoreCleanupCheckpoint {
    Retained(WorthQueryRetainedManagedGraphExecution),
    Released {
        release: WorthQueryProviderCheckpointReleaseEvidence,
        memory: WorthQueryGraphProviderMemoryArena,
    },
}

impl WorthQueryManagedGraphRestoreRecoveryRequired {
    pub fn retained(
        kind: WorthQueryManagedGraphRestoreRecoveryKind,
        detail: impl Into<Arc<str>>,
        retained: WorthQueryRetainedManagedGraphExecution,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            resource: WorthQueryManagedGraphRestoreRecoveryResource::Retained(retained),
        }
    }

    pub fn retained_after_restored_release(
        kind: WorthQueryManagedGraphRestoreRecoveryKind,
        detail: impl Into<Arc<str>>,
        retained: WorthQueryRetainedManagedGraphExecution,
        restored_execution: WorthQueryProviderExecutionReleaseEvidence,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            resource: WorthQueryManagedGraphRestoreRecoveryResource::RetainedAfterRestoredRelease {
                retained,
                restored_execution,
            },
        }
    }

    pub fn released(
        kind: WorthQueryManagedGraphRestoreRecoveryKind,
        detail: impl Into<Arc<str>>,
        checkpoint: WorthQueryProviderCheckpointReleaseEvidence,
        restored_execution: WorthQueryProviderExecutionReleaseEvidence,
        memory: WorthQueryGraphProviderMemoryArena,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            resource: WorthQueryManagedGraphRestoreRecoveryResource::Released {
                memory,
                checkpoint,
                restored_execution,
            },
        }
    }

    pub const fn kind(&self) -> WorthQueryManagedGraphRestoreRecoveryKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn checkpoint_evidence(&self) -> &WorthQueryProviderCheckpointEvidence {
        match &self.resource {
            WorthQueryManagedGraphRestoreRecoveryResource::Retained(retained)
            | WorthQueryManagedGraphRestoreRecoveryResource::RetainedAfterRestoredRelease {
                retained,
                ..
            } => retained.checkpoint_evidence(),
            WorthQueryManagedGraphRestoreRecoveryResource::Released { checkpoint, .. } => {
                checkpoint.checkpoint()
            }
        }
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        match &self.resource {
            WorthQueryManagedGraphRestoreRecoveryResource::Released { checkpoint, .. } => {
                Some(checkpoint)
            }
            WorthQueryManagedGraphRestoreRecoveryResource::Retained(_)
            | WorthQueryManagedGraphRestoreRecoveryResource::RetainedAfterRestoredRelease {
                ..
            } => None,
        }
    }

    pub const fn restored_execution_release_evidence(
        &self,
    ) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        match &self.resource {
            WorthQueryManagedGraphRestoreRecoveryResource::Retained(_) => None,
            WorthQueryManagedGraphRestoreRecoveryResource::RetainedAfterRestoredRelease {
                restored_execution,
                ..
            }
            | WorthQueryManagedGraphRestoreRecoveryResource::Released {
                restored_execution, ..
            } => Some(restored_execution),
        }
    }

    pub fn into_cleanup(self) -> WorthQueryManagedGraphRestoreCleanupRequired {
        match self.resource {
            WorthQueryManagedGraphRestoreRecoveryResource::Retained(retained) => {
                WorthQueryManagedGraphRestoreCleanupRequired::retained(retained, None)
            }
            WorthQueryManagedGraphRestoreRecoveryResource::RetainedAfterRestoredRelease {
                retained,
                restored_execution,
            } => WorthQueryManagedGraphRestoreCleanupRequired::retained(
                retained,
                Some(restored_execution),
            ),
            WorthQueryManagedGraphRestoreRecoveryResource::Released {
                checkpoint,
                restored_execution,
                memory,
            } => WorthQueryManagedGraphRestoreCleanupRequired {
                checkpoint: WorthQueryManagedGraphRestoreCleanupCheckpoint::Released {
                    release: checkpoint,
                    memory,
                },
                restored_execution_release: Some(restored_execution),
            },
        }
    }
}

impl WorthQueryManagedGraphRestoreCleanupRequired {
    pub fn retained(
        retained: WorthQueryRetainedManagedGraphExecution,
        restored_execution_release: Option<WorthQueryProviderExecutionReleaseEvidence>,
    ) -> Self {
        Self {
            checkpoint: WorthQueryManagedGraphRestoreCleanupCheckpoint::Retained(retained),
            restored_execution_release,
        }
    }

    pub fn finish(self) -> WorthQueryManagedGraphRestoreCleanupEvidence {
        let (checkpoint_release, memory) = match self.checkpoint {
            WorthQueryManagedGraphRestoreCleanupCheckpoint::Retained(retained) => {
                let memory = retained.memory.clone();
                (retained.release(), memory)
            }
            WorthQueryManagedGraphRestoreCleanupCheckpoint::Released { release, memory } => {
                (release, memory)
            }
        };
        WorthQueryManagedGraphRestoreCleanupEvidence {
            memory,
            checkpoint_release,
            restored_execution_release: self.restored_execution_release,
        }
    }
}
