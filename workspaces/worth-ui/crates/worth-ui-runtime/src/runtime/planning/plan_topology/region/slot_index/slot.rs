use super::WorthUiPlanRegionIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPlanRegionHandle {
    region_identity: WorthUiPlanRegionIdentity,
    stable_slot: u64,
    slot_generation: u64,
}

impl WorthUiPlanRegionHandle {
    pub(crate) fn initial(region_identity: WorthUiPlanRegionIdentity, stable_slot: u64) -> Self {
        Self {
            region_identity,
            stable_slot,
            slot_generation: 0,
        }
    }

    pub(crate) fn replacement_successor(
        &self,
    ) -> Result<Self, crate::runtime::WorthUiHandleCapacityExhaustion> {
        Ok(Self {
            region_identity: self.region_identity.clone(),
            stable_slot: self.stable_slot,
            slot_generation:
                crate::runtime::execution::handle_allocation::WorthUiHandleCapacity::next_slot_generation(
                    self.slot_generation,
                )?,
        })
    }

    pub(crate) fn region_identity(&self) -> &WorthUiPlanRegionIdentity {
        &self.region_identity
    }

    pub fn stable_slot(&self) -> u64 {
        self.stable_slot
    }

    pub fn slot_generation(&self) -> u64 {
        self.slot_generation
    }
}
