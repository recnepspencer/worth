//! Root-independent evacuation copying. Final publication is a separate owner
//! transition; an unfinished durable intent is never cancelled by dropping it.
mod advance;
mod begin;
mod copying;
mod evidence;
mod failure_disposition;
mod obligation;
mod resolution;
mod retention;
pub(super) use failure_disposition::CopyFailureObligation;
mod reopen;
mod session;
mod source;
mod verification;
pub(super) use obligation::SharedCopyDestination;
pub(super) use obligation::SharedCopyObligation;
pub use resolution::PhysicalExtentCopyResolutionProgress;
mod adoption;
pub use advance::{PhysicalExtentCopyPhase, PhysicalExtentCopyProgress};

use crate::physical_runtime::durability::DurableMaintenanceReceipt;
use crate::physical_runtime::record_serving::arena::ArenaReservation;
use crate::physical_runtime::stability::PhysicalRootReadLease;
pub(in crate::physical_runtime) use evidence::{
    ExtentCopySynchronization, ExtentCopyWriteEvidence,
};
pub(super) use session::CopyProducer;
pub(super) use session::ExtentCopySession;
use worth_store_physical_format::PhysicalExtentCopyIntent;

/// Single-use runtime capability, constructed only after every exact copy
/// frame and both arena synchronization effects have settled successfully.
pub(in crate::physical_runtime) struct CompletedExtentCopy {
    intent: PhysicalExtentCopyIntent,
    durable: DurableMaintenanceReceipt,
    writes: ExtentCopyWriteEvidence,
    synchronization: ExtentCopySynchronization,
    source_lease: PhysicalRootReadLease,
    reservation: ArenaReservation,
    obligation: obligation::CopyCapabilityHold,
}

impl CompletedExtentCopy {
    pub(in crate::physical_runtime::record_serving) fn into_adoption(
        self,
    ) -> (AdoptedExtentCopy, ArenaReservation) {
        (
            AdoptedExtentCopy {
                intent: self.intent,
                durable: self.durable,
                writes: self.writes,
                synchronization: self.synchronization,
                source_lease: self.source_lease,
                obligation: self.obligation,
            },
            self.reservation,
        )
    }
    pub(in crate::physical_runtime) const fn intent(&self) -> PhysicalExtentCopyIntent {
        self.intent
    }
    pub(in crate::physical_runtime) fn durable_intent_lsn(&self) -> u64 {
        self.durable.interval().2
    }
    pub(in crate::physical_runtime) fn intent_digest(&self) -> [u8; 32] {
        self.durable.payload_digest()
    }
    pub(in crate::physical_runtime) fn writes(&self) -> &ExtentCopyWriteEvidence {
        &self.writes
    }
    pub(in crate::physical_runtime) fn synchronization(&self) -> &ExtentCopySynchronization {
        &self.synchronization
    }
    pub(in crate::physical_runtime) fn validate_adoption(
        &self,
        current: worth_store_physical_format::DurableExtentRecordPlacement,
        runtime: crate::physical_runtime::RuntimeIdentity,
    ) -> Result<(), ()> {
        self.source_lease.require_live().map_err(|_| ())?;
        self.obligation.require_live()?;
        if current != self.intent.source()
            || self.writes.last_work().runtime() != runtime
            || self.reservation.range() != self.intent.destination().arena_range()
            || !self.synchronization.matches_writes(&self.writes)
            || !self.reservation.is_live()
        {
            return Err(());
        }
        Ok(())
    }
}

/// Settled-copy authority retained through final WAL and root publication.
/// The destination reservation travels separately in the root projection.
pub(in crate::physical_runtime) struct AdoptedExtentCopy {
    intent: PhysicalExtentCopyIntent,
    durable: DurableMaintenanceReceipt,
    writes: ExtentCopyWriteEvidence,
    synchronization: ExtentCopySynchronization,
    source_lease: PhysicalRootReadLease,
    obligation: obligation::CopyCapabilityHold,
}

impl AdoptedExtentCopy {
    pub(in crate::physical_runtime) const fn intent(&self) -> PhysicalExtentCopyIntent {
        self.intent
    }
    pub(in crate::physical_runtime) fn durable_intent_lsn(&self) -> u64 {
        self.durable.interval().2
    }
    pub(in crate::physical_runtime) fn intent_digest(&self) -> [u8; 32] {
        self.durable.payload_digest()
    }
    pub(in crate::physical_runtime) fn writes(&self) -> &ExtentCopyWriteEvidence {
        &self.writes
    }
    pub(in crate::physical_runtime) fn synchronization(&self) -> &ExtentCopySynchronization {
        &self.synchronization
    }
    pub(in crate::physical_runtime) fn require_live_source(&self) -> Result<(), ()> {
        self.obligation.require_live()?;
        self.source_lease.require_live().map_err(|_| ())
    }
    pub(in crate::physical_runtime) fn bind_publication(&self, lsn: u64) -> Result<(), ()> {
        self.obligation.bind_publication(lsn)
    }
    pub(in crate::physical_runtime) fn release_publication_before_effect(
        &self,
        lsn: u64,
    ) -> Result<(), ()> {
        self.obligation.release_publication_before_effect(lsn)
    }
}
