//! One in-process reader loan on a durable pin.
use worth_store_physical_format::DurablePhysicalRootManifest;

use crate::physical_runtime::{
    PhysicalReadProtectionDenial, PhysicalRecordReader, ServingPhysicalRuntime,
};
use crate::pin_table::{RootPinDenial, RootPinId};

/// Holds its pin's root reachable; dropping it releases the loan. A drained
/// `Expiring` pin completes at the next `expire` or `settle_drained`.
pub struct PhysicalRootPinLoan<'runtime> {
    serving: &'runtime ServingPhysicalRuntime,
    pin: RootPinId,
    manifest: Option<DurablePhysicalRootManifest>,
    payload: Vec<u8>,
}

impl<'runtime> PhysicalRootPinLoan<'runtime> {
    pub(super) fn admit(
        serving: &'runtime ServingPhysicalRuntime,
        pin: RootPinId,
    ) -> Result<Self, RootPinDenial> {
        let (manifest, payload) = serving.root_protection().pins().loan(pin)?;
        Ok(Self {
            serving,
            pin,
            manifest,
            payload,
        })
    }

    pub const fn pin(&self) -> RootPinId {
        self.pin
    }

    /// The owner bytes recorded with the pin.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// A protected reader of the pinned root itself, never the current root.
    pub fn records(&self) -> Result<PhysicalRecordReader, PhysicalReadProtectionDenial> {
        let manifest = self
            .manifest
            .clone()
            .ok_or(PhysicalReadProtectionDenial::MetadataUnavailable)?;
        self.serving.pinned_root_reader(manifest)
    }
}

impl std::fmt::Debug for PhysicalRootPinLoan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalRootPinLoan")
            .field("pin", &self.pin)
            .finish_non_exhaustive()
    }
}

impl Drop for PhysicalRootPinLoan<'_> {
    fn drop(&mut self) {
        self.serving.root_protection().pins().release_loan(self.pin);
    }
}
