//! One serving branch owns its derived registry and excludes pending publications.
use super::{
    readmission::{backing, Program},
    BridgeAuthoredSection,
};
use crate::conditional_execution::{
    lowering_registry::BridgeConditionalLoweringRegistry, retention::BridgeRetentionLedger,
    BridgeInstalledConditionalLowering,
};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use worth_foundational::facade::{DurableSignalBranchIdentity, SignalBranchAdvanceIdentity};
use worth_signal::facade::NodeId;
mod correspondence_posture;

pub struct BridgeReadmittedAuthoredAssembly {
    pub(in crate::conditional_execution) home_retirement: Mutex<
        Option<
            crate::conditional_execution::authored_home_retirement::PreparedAuthoredHomeRetirement,
        >,
    >,
    pub(in crate::conditional_execution) program: Arc<Program>,
    pub(in crate::conditional_execution) signal:
        worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>,
    pub(in crate::conditional_execution) branches: Arc<super::branch_registry::BranchRegistry>,
    pub(in crate::conditional_execution) _memory:
        Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::conditional_execution) struct BranchProjection {
    pub(in crate::conditional_execution) state: Mutex<Box<ReadmittedBranch>>,
}
pub(in crate::conditional_execution) enum BranchPosture {
    Serving,
    Stopped,
    ForkPreparing {
        target: DurableSignalBranchIdentity,
    },
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
    pub(in crate::conditional_execution) programme_memory:
        Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::conditional_execution) graph_lifecycle:
        worth_signal::facade::SignalGraphLifecycleProbe,
    pub(in crate::conditional_execution) posture: BranchPosture,
    pub(in crate::conditional_execution) _dependencies:
        crate::correspondence::AdmittedSemanticDependencyRegistry,
    pub(in crate::conditional_execution) _allocations:
        crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(in crate::conditional_execution) section: BridgeAuthoredSection,
    pub(in crate::conditional_execution) lowerings:
        Arc<std::sync::RwLock<BridgeConditionalLoweringRegistry>>,
    pub(in crate::conditional_execution) declarations: Arc<
        std::sync::RwLock<crate::conditional_execution::authored_declaration::DeclarationIndex>,
    >,
    pub(in crate::conditional_execution) source_owner:
        Arc<worth_proof::ConditionalSourceObservationOwner>,
    pub(in crate::conditional_execution) graph_instance_id: u64,
    pub(in crate::conditional_execution) signal_branch:
        worth_signal::facade::branch::SignalBranchIdentity,
    pub(in crate::conditional_execution) retention: Arc<BridgeRetentionLedger>,
    pub(in crate::conditional_execution) managed_clock_lanes: Arc<
        Mutex<
            std::collections::BTreeMap<
                Arc<str>,
                Arc<Mutex<crate::conditional_execution::managed_time::BridgeManagedClockLane>>,
            >,
        >,
    >,
    pub(in crate::conditional_execution) _backing: Arc<backing::Backing>,
}
#[derive(Debug)]
pub enum BridgeAuthoredBranchDenial {
    Stopped {
        branch: DurableSignalBranchIdentity,
    },
    RegistryContactHeld {
        branch: DurableSignalBranchIdentity,
    },
    ForkReserved {
        branch: DurableSignalBranchIdentity,
    },
    ForkPreparing {
        branch: DurableSignalBranchIdentity,
        target: DurableSignalBranchIdentity,
    },
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
    /// World borrows the same installed Bridge source; per-branch conditional
    /// projections remain owned by this assembly and its durable Signal owner.
    pub fn runtime_world_correspondence_port(
        &self,
    ) -> crate::facade::RuntimeWorldCorrespondencePort {
        let mut port = self.program.bridge.runtime_world_correspondence_port();
        port.branches = crate::correspondence::RuntimeWorldCorrespondenceBranches::Authored(
            Arc::downgrade(&self.branches),
        );
        port
    }
    pub fn diagnostics(&self) -> crate::diagnostics::BridgeDiagnosticsHandle {
        self.program.bridge.diagnostics.handle()
    }
    pub fn signal_owner(
        &self,
    ) -> &worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()> {
        &self.signal
    }
    pub fn with_authored_branch<R>(
        &self,
        branch: DurableSignalBranchIdentity,
        inspect: impl FnOnce(BridgeAuthoredBranchView<'_>) -> R,
    ) -> Result<R, BridgeAuthoredBranchDenial> {
        let projection = self.branches.projection(branch)?;
        let view = BridgeAuthoredBranchView {
            branch: projection.serving(branch)?,
        };
        Ok(inspect(view))
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
            BranchPosture::Stopped => Err(BridgeAuthoredBranchDenial::Stopped { branch }),
            BranchPosture::ForkPreparing { target } => {
                Err(BridgeAuthoredBranchDenial::ForkPreparing { branch, target })
            }
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
impl BranchProjection {
    /// The installed dependencies a correspondence admission compares with. A
    /// prepared definition successor changes them only when it completes, so
    /// they stay current while that successor is Preparing.
    pub(in crate::conditional_execution) fn installed_for_correspondence(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<MutexGuard<'_, Box<ReadmittedBranch>>, BridgeAuthoredBranchDenial> {
        let guard = self.state.try_lock().map_err(|denial| match denial {
            TryLockError::WouldBlock => {
                BridgeAuthoredBranchDenial::ProjectionContactHeld { branch }
            }
            TryLockError::Poisoned(_) => BridgeAuthoredBranchDenial::Poisoned { branch },
        })?;
        guard.posture.require_installed_correspondence(branch)?;
        Ok(guard)
    }
}
impl BridgeAuthoredBranchView<'_> {
    pub fn authored_section(&self) -> &BridgeAuthoredSection {
        &self.branch.section
    }
    /// Inspection retains a registry read contact only for this callback.
    pub fn with_conditional_definitions<R>(
        &self,
        inspect: impl FnOnce(&mut dyn Iterator<Item = Arc<BridgeInstalledConditionalLowering>>) -> R,
    ) -> R {
        let lowerings = self
            .branch
            .lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inspect(&mut lowerings.values().cloned())
    }
    pub fn conditional_retention(
        &self,
    ) -> crate::conditional_execution::BridgeConditionalRetentionObservation {
        self.branch.retention.observation()
    }
}
impl Drop for ReadmittedBranch {
    fn drop(&mut self) {
        crate::conditional_execution::managed_time::revoke_clock_lanes(&self.managed_clock_lanes);
        self.retention.close();
        for lowering in self
            .lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
        {
            lowering.lease.revoke_liveness();
        }
    }
}
