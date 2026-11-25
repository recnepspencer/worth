//! The serving runtime's durable root pin door and open-time restoration.
use worth_store_physical_format::DurablePhysicalRootManifest;

use super::ServingPhysicalRuntime;
use crate::physical_runtime::stability::durable_pins::PhysicalRootPins;
use crate::physical_runtime::{
    OrderedJournalName, PhysicalReadProtectionDenial, PhysicalRecordReader,
};
use crate::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId};

impl ServingPhysicalRuntime {
    /// Borrows the home's durable root pins.
    pub fn root_pins(&self) -> Result<PhysicalRootPins<'_>, RootPinDenial> {
        if self.parts.work_runtime.health.requires_inspection() {
            return Err(RootPinDenial::StorageUnavailable);
        }
        Ok(PhysicalRootPins::new(self))
    }

    pub(in crate::physical_runtime) fn pin_current_root(
        &self,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        payload: Vec<u8>,
    ) -> Result<(RootPinId, DurablePhysicalRootManifest), RootPinDenial> {
        self.parts
            .publication
            .pin_current_root(parent, policy, payload)
    }

    /// A protected reader of a pinned, possibly displaced, root.
    pub(in crate::physical_runtime) fn pinned_root_reader(
        &self,
        root: DurablePhysicalRootManifest,
    ) -> Result<PhysicalRecordReader, PhysicalReadProtectionDenial> {
        let lease = self.root_protection().capture(&root)?;
        Ok(self.reader_from_captured_root(root, lease))
    }

    /// Replay the pin journal before this runtime is handed out. A Store
    /// whose registry declares no pin journal holds no durable pins.
    pub(in crate::physical_runtime) fn restore_root_pins(&self) {
        let registry = self.root_protection();
        let pins = registry.pins();
        let prior = pins.take_staged();
        let name = OrderedJournalName::physical_root_pins();
        if !self.journal_registry.contains(name) {
            pins.install_restored(Default::default());
            return;
        }
        let restored = self.ordered_journal(name).ok().and_then(|journal| {
            crate::physical_runtime::stability::durable_pins::restore::replay(&journal, prior).ok()
        });
        match restored {
            Some(core) => pins.install_restored(core),
            None => pins.mark_unrestored(),
        }
    }
}
