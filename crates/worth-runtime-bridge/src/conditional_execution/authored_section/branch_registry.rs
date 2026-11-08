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
pub(in crate::conditional_execution) struct BranchRegistry {
    pub(super) slots: Mutex<BTreeMap<DurableSignalBranchIdentity, BranchSlot>>,
}
impl BranchRegistry {
    pub(in crate::conditional_execution) fn new(
        branches: BTreeMap<DurableSignalBranchIdentity, BranchSlot>,
    ) -> Arc<Self> {
        Arc::new(Self {
            slots: Mutex::new(branches),
        })
    }
    pub(super) fn projection(
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
