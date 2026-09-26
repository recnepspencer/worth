#[cfg(test)]
use super::UiAllocationFrameSourceRetirementDenial;
use super::{
    UiAllocationFrameSourceGeneration, UiAllocationFrameSourceIdentity,
    UiAllocationFrameSourceLane, UiAllocationFrameSourceLease,
    UiAllocationFrameSourceLeaseIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationFrameSourceAdmissionDenial {
    RegistryFull,
    AlreadyActive,
    StaleGeneration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiAllocationFrameSourceRegistration {
    lane: UiAllocationFrameSourceLane,
    identity: UiAllocationFrameSourceIdentity,
    generation: UiAllocationFrameSourceGeneration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiAllocationFrameSourceSlot {
    lease_generation: u64,
    registration: Option<UiAllocationFrameSourceRegistration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationFrameSourceRegistry {
    slots: [UiAllocationFrameSourceSlot; super::ALLOCATION_FRAME_SOURCE_CAPACITY],
}

impl UiAllocationFrameSourceRegistry {
    pub(crate) fn empty(runtime_generation: u64) -> Self {
        Self {
            slots: std::array::from_fn(|_| UiAllocationFrameSourceSlot {
                lease_generation: runtime_generation,
                registration: None,
            }),
        }
    }

    pub(crate) fn admit(
        &mut self,
        lane: UiAllocationFrameSourceLane,
        identity: UiAllocationFrameSourceIdentity,
        generation: UiAllocationFrameSourceGeneration,
    ) -> Result<UiAllocationFrameSourceLease, UiAllocationFrameSourceAdmissionDenial> {
        let registration = UiAllocationFrameSourceRegistration {
            lane,
            identity,
            generation,
        };
        if self.slots.iter().any(|slot| {
            slot.registration.as_ref().is_some_and(|active| {
                active.lane == registration.lane && active.identity == registration.identity
            })
        }) {
            return Err(UiAllocationFrameSourceAdmissionDenial::AlreadyActive);
        }
        let (slot_index, slot) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.registration.is_none())
            .ok_or(UiAllocationFrameSourceAdmissionDenial::RegistryFull)?;
        slot.registration = Some(registration);
        Ok(UiAllocationFrameSourceLease::from_registry(
            UiAllocationFrameSourceLeaseIdentity::from_registry(
                u16::try_from(slot_index)
                    .expect("the registry holds ALLOCATION_FRAME_SOURCE_COUNT slots"),
                slot.lease_generation,
            ),
            lane,
            identity,
            generation,
        ))
    }

    pub(crate) fn advance_generation(
        &mut self,
        lease: &UiAllocationFrameSourceLease,
        generation: UiAllocationFrameSourceGeneration,
    ) -> Result<UiAllocationFrameSourceLease, UiAllocationFrameSourceAdmissionDenial> {
        if generation <= lease.source_generation() {
            return Err(UiAllocationFrameSourceAdmissionDenial::StaleGeneration);
        }
        let slot = &mut self.slots[usize::from(lease.lease_identity().slot())];
        slot.lease_generation = slot
            .lease_generation
            .checked_add(1)
            .ok_or(UiAllocationFrameSourceAdmissionDenial::RegistryFull)?;
        slot.registration = Some(UiAllocationFrameSourceRegistration {
            lane: lease.source_lane(),
            identity: lease.source_identity(),
            generation,
        });
        Ok(UiAllocationFrameSourceLease::from_registry(
            UiAllocationFrameSourceLeaseIdentity::from_registry(
                lease.lease_identity().slot(),
                slot.lease_generation,
            ),
            lease.source_lane(),
            lease.source_identity(),
            generation,
        ))
    }

    #[cfg(test)]
    pub(crate) fn validate_retirement(
        &self,
        lease: &UiAllocationFrameSourceLease,
    ) -> Result<(), UiAllocationFrameSourceRetirementDenial> {
        let Some(slot) = self.slots.get(usize::from(lease.lease_identity().slot())) else {
            return Err(UiAllocationFrameSourceRetirementDenial::LeaseExpired);
        };
        if slot.lease_generation != lease.lease_identity().generation() {
            return Err(UiAllocationFrameSourceRetirementDenial::LeaseExpired);
        }
        let expected = UiAllocationFrameSourceRegistration {
            lane: lease.source_lane(),
            identity: lease.source_identity(),
            generation: lease.source_generation(),
        };
        if slot.registration != Some(expected) {
            return Err(UiAllocationFrameSourceRetirementDenial::LeaseExpired);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn retire_validated(&mut self, lease: &UiAllocationFrameSourceLease) {
        let slot = &mut self.slots[usize::from(lease.lease_identity().slot())];
        slot.registration = None;
        slot.lease_generation = slot
            .lease_generation
            .checked_add(1)
            .expect("lease generation exhaustion is denied during admission");
    }
}
