//! Shared projection membership reserves the exact target before durable I/O.
use super::{branch_projection::BranchProjection, BridgeAuthoredBranchDenial as D};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, TryLockError},
};
use worth_foundational::facade::{DurableSignalBranchIdentity, SignalBranchAdvanceIdentity};

pub(in crate::conditional_execution) enum BranchSlot {
    Serving {
        projection: Arc<BranchProjection>,
        _memory: Option<worth_execution::ExecutionMemoryReservation>,
    },
    Reserved {
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    },
    ReadmissionRequired(SignalBranchAdvanceIdentity),
}
pub(crate) struct BranchRegistry {
    pub(super) signals: Mutex<
        BTreeMap<worth_signal::facade::branch::SignalBranchIdentity, DurableSignalBranchIdentity>,
    >,
    pub(in crate::conditional_execution) slots:
        Mutex<BTreeMap<DurableSignalBranchIdentity, BranchSlot>>,
}
impl BranchRegistry {
    pub(in crate::conditional_execution) fn new(
        branches: BTreeMap<DurableSignalBranchIdentity, BranchSlot>,
    ) -> Arc<Self> {
        let mut signals = BTreeMap::new();
        for (branch, slot) in &branches {
            if let BranchSlot::Serving { projection, .. } = slot {
                let state = projection
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                assert!(
                    signals
                        .insert(state.signal_branch.clone(), *branch)
                        .is_none(),
                    "Signal branch identities identify one actual branch"
                );
            }
        }
        Arc::new(Self {
            signals: Mutex::new(signals),
            slots: Mutex::new(branches),
        })
    }
    pub(crate) fn with_branch_registry<R>(
        &self,
        graph: u64,
        signal_branch: &worth_signal::facade::branch::SignalBranchIdentity,
        inspect: impl FnOnce(&crate::correspondence::AdmittedSemanticDependencyRegistry) -> R,
    ) -> Result<R, crate::facade::RuntimeWorldCorrespondenceAdmissionDenial> {
        use crate::facade::RuntimeWorldCorrespondenceAdmissionDenial as Denial;
        let refusal = || Denial::AuthoredBranchUnavailable {
            signal_graph_instance_id: graph,
        };
        let branch = self
            .signals
            .try_lock()
            .map_err(|_| refusal())?
            .get(signal_branch)
            .copied()
            .ok_or_else(refusal)?;
        let projection = self.projection(branch).map_err(|_| refusal())?;
        let serving = projection
            .installed_for_correspondence(branch)
            .map_err(|_| refusal())?;
        Ok(inspect(&serving._dependencies))
    }
    pub(in crate::conditional_execution) fn projection(
        &self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<Arc<BranchProjection>, D> {
        let slots = self.slots.try_lock().map_err(|cause| match cause {
            TryLockError::WouldBlock => D::RegistryContactHeld { branch },
            TryLockError::Poisoned(_) => D::Poisoned { branch },
        })?;
        match slots.get(&branch) {
            Some(BranchSlot::Serving { projection, .. }) => Ok(Arc::clone(projection)),
            Some(BranchSlot::Reserved { .. }) => Err(D::ForkReserved { branch }),
            Some(BranchSlot::ReadmissionRequired(original)) => Err(D::ReadmissionRequired {
                branch,
                original: *original,
            }),
            None => Err(D::UnknownBranch { branch }),
        }
    }
}
