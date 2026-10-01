//! The live C8 baseline attached to the existing Store allocation owner.
//! Installing a baseline reduces available bytes; it issues no media custody.

use super::PhysicalRecoveryCoordination;

pub(super) enum RecoveryRejoinResidentState {
    Unadmitted,
    Admitted { retained: u64 },
    Consumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryRejoinResidentAdmissionDenial {
    MissingAllocation,
    AlreadyAdmitted,
    SizeOverflow,
    RecoveryMemoryBytes { observed: u64, admitted: u64 },
}

impl PhysicalRecoveryCoordination {
    pub fn admit_rejoin_resident_bytes(
        &mut self,
        already_live: u64,
    ) -> Result<(), PhysicalRecoveryRejoinResidentAdmissionDenial> {
        if !matches!(
            self.rejoin_resident,
            RecoveryRejoinResidentState::Unadmitted
        ) {
            return Err(PhysicalRecoveryRejoinResidentAdmissionDenial::AlreadyAdmitted);
        }
        let allocation = self
            .recovery_allocation
            .ok_or(PhysicalRecoveryRejoinResidentAdmissionDenial::MissingAllocation)?;
        if already_live > allocation.byte_limit() {
            return Err(
                PhysicalRecoveryRejoinResidentAdmissionDenial::RecoveryMemoryBytes {
                    observed: already_live,
                    admitted: allocation.byte_limit(),
                },
            );
        }
        self.rejoin_resident = RecoveryRejoinResidentState::Admitted {
            retained: already_live,
        };
        Ok(())
    }

    pub(in crate::physical_runtime) fn take_rejoin_resident_basis(
        &mut self,
    ) -> Option<(
        crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        u64,
    )> {
        let allocation = self.recovery_allocation?;
        match std::mem::replace(
            &mut self.rejoin_resident,
            RecoveryRejoinResidentState::Consumed,
        ) {
            RecoveryRejoinResidentState::Admitted { retained } => Some((allocation, retained)),
            RecoveryRejoinResidentState::Unadmitted | RecoveryRejoinResidentState::Consumed => None,
        }
    }
}
