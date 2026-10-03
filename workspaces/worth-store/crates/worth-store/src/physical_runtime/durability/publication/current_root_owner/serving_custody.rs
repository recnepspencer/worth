//! Serving entry funds the standing capture reservation of a selected ledger
//! that holds released-drop custody, before Serving is sealed, so a pool hold
//! taken after entry cannot starve the checkpoints that certify it. Clean
//! reopen and the C.8 rejoin handoff both enter here; an unfundable
//! reservation denies entry before any effect. A ledger with no custody holds
//! no bytes; its first drop admission establishes the reservation before the
//! drop's effects.

use super::release_capacity::{
    backing::ReleasePublicationAllocationOwner, ReleaseCertificateCapacityDenial,
    ReleaseLedgerState,
};
use super::{CheckpointCustodyOrigin, PreparedRecoveredCheckpointCustody};

pub(in crate::physical_runtime) struct ServingCheckpointCustody {
    pub(super) origin: CheckpointCustodyOrigin,
    pub(super) recovered: Option<PreparedRecoveredCheckpointCustody>,
    pub(super) release_allocation: ReleasePublicationAllocationOwner,
    pub(super) release_ledger: ReleaseLedgerState,
}

impl ServingCheckpointCustody {
    /// Reserves the capture envelope of whichever ledger Serving will hold
    /// (the recovered ledger when C.8 hands one over, else the origin's)
    /// when that ledger holds released-drop custody.
    pub(in crate::physical_runtime) fn reserve(
        origin: CheckpointCustodyOrigin,
        mut recovered: Option<PreparedRecoveredCheckpointCustody>,
        frame_ports: crate::physical_runtime::record_serving::RecordFramePorts,
        ceiling: crate::physical_runtime::PhysicalRecoveryAllocationAdmission,
        runtime: crate::physical_runtime::RuntimeIdentity,
        generation: crate::physical_runtime::LifecycleGeneration,
        lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
        capture_scan_bytes: u64,
    ) -> Result<Self, ReleaseCertificateCapacityDenial> {
        let release_allocation = ReleasePublicationAllocationOwner::new(
            frame_ports,
            ceiling,
            runtime,
            generation,
            lifecycle,
            capture_scan_bytes,
        );
        let mut release_ledger = ReleaseLedgerState::from_origin(origin);
        let serving_ledger = match recovered.as_mut() {
            Some(recovered) => recovered.release_ledger_mut(),
            None => &mut release_ledger,
        };
        if let Some(ledger) = serving_ledger
            .selected_mut()
            .filter(|ledger| ledger.holds_release_custody())
        {
            ledger.reserve_capture_custody(&release_allocation, ceiling, None)?;
        }
        Ok(Self {
            origin,
            recovered,
            release_allocation,
            release_ledger,
        })
    }
}
