//! One global observation roster, prepaid before frame projection and Arc sealing.

use super::{
    ObservationData, ObservationStorage, PhysicalRecoveryIntegrityObservations,
    PhysicalRecoveryWalIntegrityObservation,
};
use crate::orchestration::NativeWalRoster;
use std::{
    alloc::Layout,
    sync::{atomic::AtomicUsize, Arc},
};
use worth_store::physical_runtime::{PhysicalRecoveryCoordination, RecoveryWalAllocationDenial};

pub(crate) struct WalIntegrityObservationBuilder {
    roster: NativeWalRoster<PhysicalRecoveryWalIntegrityObservation>,
}

impl WalIntegrityObservationBuilder {
    pub(crate) fn new() -> Self {
        let (layout, _) = Layout::new::<[AtomicUsize; 2]>()
            .extend(Layout::new::<ObservationData>())
            .expect("fixed observation Arc layout fits address space");
        Self {
            roster: NativeWalRoster::empty(layout.pad_to_align().size() as u64),
        }
    }

    pub(crate) fn reserve_one(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
    ) -> Result<(), RecoveryWalAllocationDenial> {
        self.roster.reserve_one(owner)
    }

    pub(crate) fn push_reserved(&mut self, observation: PhysicalRecoveryWalIntegrityObservation) {
        self.roster.push_reserved(observation);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.roster.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.roster.is_empty()
    }

    pub(crate) fn finish(self) -> PhysicalRecoveryIntegrityObservations {
        if self.is_empty() {
            return PhysicalRecoveryIntegrityObservations::default();
        }
        PhysicalRecoveryIntegrityObservations {
            storage: ObservationStorage::Shared(Arc::new(ObservationData {
                roster: self.roster,
            })),
        }
    }
}
