//! Memory-Home parity: World's retention port over Store's volatile pin table,
//! the identical verified `Active -> Expiring -> Expired` and nested-overlay
//! state machine, held for the live Home's lifetime.
use std::sync::Arc;
use worth_runtime_world::facade::{
    FrontierPinCustody, HomeRootManifest, LeaseTransition, PinId, PinSpec, PinnedFrontierLease,
    RetentionPortDenial, RuntimeWorldRetentionPort,
};
use worth_store::pin_table::VolatileRootPinTable;

use super::{denial, lease_state, policy, root_pin};

type FrontierSource = dyn Fn() -> HomeRootManifest + Send + Sync;

/// Frontier pins of a memory Home. The frontier source returns the live
/// Home's current sealed frontier.
#[derive(Clone)]
pub struct VolatileHomeRetention {
    table: Arc<VolatileRootPinTable>,
    frontier: Arc<FrontierSource>,
}

impl std::fmt::Debug for VolatileHomeRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VolatileHomeRetention")
            .field("table", &self.table)
            .finish_non_exhaustive()
    }
}

impl VolatileHomeRetention {
    pub fn new(frontier: impl Fn() -> HomeRootManifest + Send + Sync + 'static) -> Self {
        Self {
            table: Arc::new(VolatileRootPinTable::default()),
            frontier: Arc::new(frontier),
        }
    }

    /// Whether any live pin protects the frontier with this sequence.
    pub fn protects_frontier(&self, frontier_sequence: u64) -> bool {
        self.table.protects_root(frontier_sequence)
    }
}

impl RuntimeWorldRetentionPort for VolatileHomeRetention {
    fn capture_and_pin_frontier(
        &self,
        spec: PinSpec,
    ) -> Result<PinnedFrontierLease, RetentionPortDenial> {
        let manifest = (self.frontier)();
        let pin = self
            .table
            .pin(
                manifest.frontier_sequence(),
                spec.parent_pin.map(root_pin),
                policy(spec.nested_drop_policy),
                manifest.encode(),
            )
            .map_err(denial)?;
        let pin_id = PinId::from_retention_owner(pin.ordinal());
        Ok(PinnedFrontierLease::from_retention_owner(
            pin_id,
            manifest,
            Box::new(VolatileFrontierCustody {
                retention: self.clone(),
                pin: pin_id,
            }),
        ))
    }

    fn expire_lease(&self, pin: PinId) -> Result<LeaseTransition, RetentionPortDenial> {
        let state = self.table.expire(root_pin(pin)).map_err(denial)?;
        Ok(LeaseTransition::new(pin, lease_state(state)))
    }

    fn lease_state(&self, pin: PinId) -> Option<worth_runtime_world::facade::LeaseState> {
        self.table.state(root_pin(pin)).map(lease_state)
    }

    fn pinned_manifest(&self, pin: PinId) -> Result<HomeRootManifest, RetentionPortDenial> {
        let bytes = self
            .table
            .payload(root_pin(pin))
            .ok_or(RetentionPortDenial::PinNotFound)?;
        HomeRootManifest::decode(&bytes).ok_or(RetentionPortDenial::CorruptManifest)
    }

    /// Volatile pins complete synchronously as loans and children drain.
    fn settle_drained(&self) -> Result<usize, RetentionPortDenial> {
        Ok(0)
    }
}

#[derive(Debug)]
struct VolatileFrontierCustody {
    retention: VolatileHomeRetention,
    pin: PinId,
}

impl FrontierPinCustody for VolatileFrontierCustody {
    fn read_pinned_manifest(&self) -> Result<HomeRootManifest, RetentionPortDenial> {
        let pin = root_pin(self.pin);
        let bytes = self.retention.table.loan(pin).map_err(denial)?;
        self.retention.table.release_loan(pin);
        HomeRootManifest::decode(&bytes).ok_or(RetentionPortDenial::CorruptManifest)
    }

    fn release(&self) {
        let _ = self.retention.expire_lease(self.pin);
    }
}
