use crate::physical_runtime::durability::PhysicalMutationUnresolvedBindingObservation;
use crate::physical_runtime::record_serving::arena::ArenaReservationObligation;
use crate::physical_runtime::stability::PhysicalRootReadLease;
use std::sync::{Arc, Mutex};
use worth_store_physical_format::PhysicalExtentCopyIntent;
mod recovered;
pub(super) use recovered::CopyResolution;
pub(in crate::physical_runtime::record_serving::publication::director) use recovered::{
    CopyDestination, SharedCopyDestination,
};

/// Director ownership survives every transient prepared/adopted copy carrier.
pub(in crate::physical_runtime::record_serving::publication::director) type SharedCopyObligation =
    Arc<Mutex<CopyObligation>>;

pub(super) enum CopyBinding {
    Live(PhysicalMutationUnresolvedBindingObservation),
    /// No final publication WAL binding exists. Reopen may abandon this copy
    /// obligation, but cannot invent the original unsealed admission.
    RecoveredUnpublished,
}

pub(in crate::physical_runtime::record_serving::publication::director) struct CopyObligation {
    pub(super) operation: [u8; 32],
    pub(super) source_lease: PhysicalRootReadLease,
    pub(super) reservation: SharedCopyDestination,
    // Physical candidate space is not WAL metadata. It transfers to the
    // displaced source at publication, or remains until durable cancellation.
    pub(super) physical_growth: Option<crate::physical_runtime::durability::RetainedByteLease>,
    pub(super) binding: CopyBinding,
    pub(super) intent: Option<(PhysicalExtentCopyIntent, u64, [u8; 32])>,
    pub(super) carrier_alive: bool,
    pub(super) publication_lsn: Option<u64>,
    pub(super) resolved: bool,
    pub(super) published_root: Option<u64>,
    pub(super) resolution: Option<CopyResolution>,
    pub(super) inspection: bool,
    pub(super) resolution_requested:
        Option<worth_store_physical_format::PhysicalExtentCopyResolutionKind>,
}

/// Dropping this carrier does not drop the director's obligation. It merely
/// makes durable cancellation possible when no final publication has escaped.
pub(super) struct CopyCapabilityHold {
    obligation: SharedCopyObligation,
}

impl CopyCapabilityHold {
    pub(super) fn issue(obligation: &SharedCopyObligation) -> Result<Self, ()> {
        let mut state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.carrier_alive
            || state.resolved
            || state.resolution_requested.is_some()
            || state.inspection
        {
            return Err(());
        }
        state.carrier_alive = true;
        Ok(Self {
            obligation: Arc::clone(obligation),
        })
    }
    pub(super) fn bind_publication(&self, lsn: u64) -> Result<(), ()> {
        let mut state = self.obligation.lock().unwrap_or_else(|e| e.into_inner());
        let (_, intent_lsn, _) = state.intent.ok_or(())?;
        if state.resolved
            || state.resolution_requested.is_some()
            || state.inspection
            || !state.carrier_alive
            || lsn <= intent_lsn
            || state.publication_lsn.is_some_and(|prior| prior != lsn)
        {
            return Err(());
        }
        state.publication_lsn = Some(lsn);
        Ok(())
    }
    pub(super) fn require_live(&self) -> Result<(), ()> {
        let state = self.obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.resolved
            || !state.carrier_alive
            || state.resolution_requested.is_some()
            || state.inspection
        {
            return Err(());
        }
        state.source_lease.require_live().map_err(|_| ())
    }
    pub(super) fn release_publication_before_effect(&self, lsn: u64) -> Result<(), ()> {
        let mut state = self.obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.publication_lsn != Some(lsn)
            || state.published_root.is_some()
            || state.resolved
            || state.resolution_requested.is_some()
            || state.inspection
        {
            return Err(());
        }
        state.publication_lsn = None;
        Ok(())
    }
}
impl Drop for CopyCapabilityHold {
    fn drop(&mut self) {
        self.obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .carrier_alive = false;
    }
}
