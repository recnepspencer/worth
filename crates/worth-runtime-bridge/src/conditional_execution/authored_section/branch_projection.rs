//! One serving branch owns its derived registry and excludes pending publications.
use super::{
    readmission::{backing, Program},
    BridgeAuthoredSection,
};
use crate::conditional_execution::{
    lowering_registry::BridgeConditionalLoweringRegistry, retention::BridgeRetentionLedger,
    BridgeInstalledConditionalLowering,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard, TryLockError},
};
use worth_foundational::facade::{DurableSignalBranchIdentity, SignalBranchAdvanceIdentity};
use worth_signal::facade::NodeId;

pub struct BridgeReadmittedAuthoredAssembly {
    pub(in crate::conditional_execution) program: Arc<Program>,
    pub(in crate::conditional_execution) signal:
        worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>,
    pub(in crate::conditional_execution) branches:
        BTreeMap<DurableSignalBranchIdentity, Arc<BranchProjection>>,
    pub(in crate::conditional_execution) _memory:
        Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::conditional_execution) struct BranchProjection {
    pub(in crate::conditional_execution) state: Mutex<Box<ReadmittedBranch>>,
}
pub(in crate::conditional_execution) enum BranchPosture {
    Serving,
    Preparing {
        node: NodeId,
    },
    Pending {
        original: SignalBranchAdvanceIdentity,
    },
    ReadmissionRequired {
        original: SignalBranchAdvanceIdentity,
    },
}
pub(in crate::conditional_execution) struct ReadmittedBranch {
    pub(in crate::conditional_execution) posture: BranchPosture,
    pub(in crate::conditional_execution) _dependencies:
        crate::correspondence::AdmittedSemanticDependencyRegistry,
    pub(in crate::conditional_execution) _allocations:
        crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(in crate::conditional_execution) section: BridgeAuthoredSection,
    pub(in crate::conditional_execution) lowerings: BridgeConditionalLoweringRegistry,
    pub(in crate::conditional_execution) retention: Arc<BridgeRetentionLedger>,
    pub(in crate::conditional_execution) _backing: Arc<backing::Backing>,
    pub(in crate::conditional_execution) _program: Arc<Program>,
}
#[derive(Debug)]
pub enum BridgeAuthoredBranchDenial {
    UnknownBranch {
        branch: DurableSignalBranchIdentity,
    },
    ProjectionContactHeld {
        branch: DurableSignalBranchIdentity,
    },
    Preparing {
        branch: DurableSignalBranchIdentity,
        node: NodeId,
    },
    Pending {
        branch: DurableSignalBranchIdentity,
        original: SignalBranchAdvanceIdentity,
    },
    ReadmissionRequired {
        branch: DurableSignalBranchIdentity,
        original: SignalBranchAdvanceIdentity,
    },
    Poisoned {
        branch: DurableSignalBranchIdentity,
    },
}
pub struct BridgeAuthoredBranchView<'a> {
    branch: MutexGuard<'a, Box<ReadmittedBranch>>,
}
impl BridgeReadmittedAuthoredAssembly {
    pub fn diagnostics(&self) -> crate::diagnostics::BridgeDiagnosticsHandle {
        self.program.bridge.diagnostics.handle()
    }
    pub fn signal_owner(
        &self,
    ) -> &worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()> {
        &self.signal
    }
    pub fn authored_branch(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<BridgeAuthoredBranchView<'_>, BridgeAuthoredBranchDenial> {
        let projection = self
            .branches
            .get(&branch)
            .ok_or(BridgeAuthoredBranchDenial::UnknownBranch { branch })?;
        Ok(BridgeAuthoredBranchView {
            branch: projection.serving(branch)?,
        })
    }
}
impl BranchProjection {
    pub(in crate::conditional_execution) fn serving(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<MutexGuard<'_, Box<ReadmittedBranch>>, BridgeAuthoredBranchDenial> {
        let guard = self.state.try_lock().map_err(|denial| match denial {
            TryLockError::WouldBlock => {
                BridgeAuthoredBranchDenial::ProjectionContactHeld { branch }
            }
            TryLockError::Poisoned(_) => BridgeAuthoredBranchDenial::Poisoned { branch },
        })?;
        match guard.posture {
            BranchPosture::Serving => Ok(guard),
            BranchPosture::ReadmissionRequired { original } => {
                Err(BridgeAuthoredBranchDenial::ReadmissionRequired { branch, original })
            }
            BranchPosture::Preparing { node } => {
                Err(BridgeAuthoredBranchDenial::Preparing { branch, node })
            }
            BranchPosture::Pending { original } => {
                Err(BridgeAuthoredBranchDenial::Pending { branch, original })
            }
        }
    }
}
impl BridgeAuthoredBranchView<'_> {
    pub fn authored_section(&self) -> &BridgeAuthoredSection {
        &self.branch.section
    }
    pub fn conditional_definitions(
        &self,
    ) -> impl Iterator<Item = Arc<BridgeInstalledConditionalLowering>> + '_ {
        self.branch.lowerings.values().cloned()
    }
    pub fn conditional_retention(
        &self,
    ) -> crate::conditional_execution::BridgeConditionalRetentionObservation {
        self.branch.retention.observation()
    }
}
impl Drop for ReadmittedBranch {
    fn drop(&mut self) {
        self.retention.close();
        for lowering in self.lowerings.values() {
            lowering.lease.revoke_liveness();
        }
    }
}
