use std::sync::{Arc, Mutex};

use super::{
    BridgeManagedClockBinding, BridgeManagedClockInstallationParts, BridgeManagedTemporalDenial,
    BridgeManagedTemporalDenialKind,
};
use crate::conditional_execution::{BridgeInstalledConditionalLowering, BridgeOwnedSignalRuntime};

impl BridgeOwnedSignalRuntime {
    pub fn install_managed_clock(
        &self,
        parts: BridgeManagedClockInstallationParts<'_>,
    ) -> Result<BridgeManagedClockBinding, BridgeManagedTemporalDenial> {
        self.require_managed_clock_lowering(parts.lowering)?;
        let declaration = super::BridgeManagedClockDeclaration::admit(
            self.bridge.signal_runtime_key,
            &self.retention,
            parts,
        )?;
        let identity = Arc::clone(declaration.identity());
        let binding = declaration.binding();
        if self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&identity)
        {
            return Err(duplicate_clock());
        }
        // Owner issuance happens after the registry lookup has released its lock.
        let lane = Arc::new(Mutex::new(declaration.seal()?));
        let mut lanes = self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let std::collections::btree_map::Entry::Vacant(entry) = lanes.entry(identity) {
            entry.insert(lane);
            Ok(binding)
        } else {
            drop(lanes);
            drop(lane);
            Err(duplicate_clock())
        }
    }

    pub(in crate::conditional_execution) fn require_managed_clock_lowering(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<(), BridgeManagedTemporalDenial> {
        self.require_live_installed_lowering(lowering)
            .map_err(|denial| {
                BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::ForeignClockBinding,
                    denial.detail(),
                )
            })
    }
}

fn duplicate_clock() -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::DuplicateClockBinding,
        "managed clock binding identity is already installed",
    )
}
