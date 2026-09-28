use super::UiIntentOccupancyPosture;

const UI_INTENT_OCCUPANCY_CAPACITY: usize = super::super::admission::UI_INTENT_ADMISSION_CAPACITY;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiIntentOccupancyKey {
    TargetRoute {
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        declaration: crate::declaration::UiIntentDeclarationIdentity,
    },
    Declaration(crate::declaration::UiIntentDeclarationIdentity),
    Definition(crate::capability::UiIntentId),
    Application,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiIntentOccupancyObservation {
    key: UiIntentOccupancyKey,
    posture: UiIntentOccupancyPosture,
}

pub(crate) struct UiIntentOccupancyState {
    slots: [UiIntentOccupancySlot; UI_INTENT_OCCUPANCY_CAPACITY],
}

struct UiIntentOccupancySlot {
    generation: u64,
    key: Option<UiIntentOccupancyKey>,
}

pub struct UiIntentOccupancyReservation {
    slot: u8,
    generation: u64,
}

pub(crate) struct UiIntentOccupancyPlacement {
    slot: u8,
    generation: u64,
    slots_inspected: usize,
}

pub(crate) struct UiIntentOccupancyPreparationFailure {
    denial: UiIntentOccupancyReservationDenial,
    slots_inspected: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentOccupancyReservationDenial {
    ScopeBecameOccupied,
    CapacityExceeded { maximum: usize },
    ReservationIdentityExhausted,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentOccupancyReleasePosture {
    Released,
    CancelledByLifecycle,
}

impl UiIntentOccupancyState {
    pub(crate) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| UiIntentOccupancySlot {
                generation: 0,
                key: None,
            }),
        }
    }

    pub(crate) fn observe(
        &self,
        scope: crate::declaration::UiIntentConcurrencyScope,
        declaration: &crate::declaration::UiCanonicalIntentDeclaration,
        definition: crate::capability::UiIntentId,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    ) -> UiIntentOccupancyObservation {
        let key = occupancy_key(scope, declaration, definition, target);
        let posture = self.posture_for(&key);
        UiIntentOccupancyObservation { key, posture }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn reserve(
        &mut self,
        proof: super::UiIntentOperabilityProof,
    ) -> Result<UiIntentOccupancyReservation, UiIntentOccupancyReservationDenial> {
        let observation = proof.occupancy_observation();
        let placement = self
            .prepare_observation_reservation(observation)
            .map_err(|failure| failure.denial)?;
        Ok(self.commit_observation_reservation(observation, placement))
    }

    pub(crate) fn prepare_candidate_reservation(
        &self,
        candidate: &super::super::admission::UiCurrentIntentAdmissionCandidate,
    ) -> Result<UiIntentOccupancyPlacement, UiIntentOccupancyPreparationFailure> {
        self.prepare_observation_reservation(candidate.occupancy_observation())
    }

    pub(crate) fn commit_candidate_reservation(
        &mut self,
        candidate: &super::super::admission::UiCurrentIntentAdmissionCandidate,
        placement: UiIntentOccupancyPlacement,
    ) -> UiIntentOccupancyReservation {
        self.commit_observation_reservation(candidate.occupancy_observation(), placement)
    }

    pub(crate) fn release(
        &mut self,
        reserved: UiIntentOccupancyReservation,
    ) -> UiIntentOccupancyReleasePosture {
        let slot = &mut self.slots[reserved.slot as usize];
        if slot.generation == reserved.generation && slot.key.is_some() {
            slot.key = None;
            UiIntentOccupancyReleasePosture::Released
        } else {
            UiIntentOccupancyReleasePosture::CancelledByLifecycle
        }
    }

    pub(crate) fn active_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.key.is_some()).count()
    }

    pub(crate) fn is_current_observation(
        &self,
        observation: &UiIntentOccupancyObservation,
    ) -> bool {
        self.posture_for(&observation.key) == observation.posture
    }

    fn prepare_observation_reservation(
        &self,
        observation: &UiIntentOccupancyObservation,
    ) -> Result<UiIntentOccupancyPlacement, UiIntentOccupancyPreparationFailure> {
        if observation.posture != UiIntentOccupancyPosture::Idle {
            return Err(UiIntentOccupancyPreparationFailure::new(
                UiIntentOccupancyReservationDenial::ScopeBecameOccupied,
                0,
            ));
        }
        let mut vacant = None;
        let mut inspected = 0;
        for (index, slot) in self.slots.iter().enumerate() {
            inspected += 1;
            if slot.key.as_ref() == Some(&observation.key) {
                return Err(UiIntentOccupancyPreparationFailure::new(
                    UiIntentOccupancyReservationDenial::ScopeBecameOccupied,
                    inspected,
                ));
            }
            if vacant.is_none() && slot.key.is_none() {
                vacant = Some(index);
            }
        }
        let slot_index = vacant.ok_or_else(|| {
            UiIntentOccupancyPreparationFailure::new(
                UiIntentOccupancyReservationDenial::CapacityExceeded {
                    maximum: UI_INTENT_OCCUPANCY_CAPACITY,
                },
                inspected,
            )
        })?;
        let slot = &self.slots[slot_index];
        let generation = slot.generation.checked_add(1).ok_or_else(|| {
            UiIntentOccupancyPreparationFailure::new(
                UiIntentOccupancyReservationDenial::ReservationIdentityExhausted,
                inspected,
            )
        })?;
        Ok(UiIntentOccupancyPlacement {
            slot: u8::try_from(slot_index).expect("UI_INTENT_OCCUPANCY_CAPACITY slots fit u8"),
            generation,
            slots_inspected: inspected,
        })
    }

    fn commit_observation_reservation(
        &mut self,
        observation: &UiIntentOccupancyObservation,
        placement: UiIntentOccupancyPlacement,
    ) -> UiIntentOccupancyReservation {
        let slot = &mut self.slots[placement.slot as usize];
        assert_eq!(
            slot.generation.checked_add(1),
            Some(placement.generation),
            "sealed occupancy placement generation remains current"
        );
        assert!(
            slot.key.is_none(),
            "sealed occupancy placement remains vacant"
        );
        slot.generation = placement.generation;
        slot.key = Some(observation.key.clone());
        UiIntentOccupancyReservation {
            slot: placement.slot,
            generation: placement.generation,
        }
    }

    fn posture_for(&self, key: &UiIntentOccupancyKey) -> UiIntentOccupancyPosture {
        if self.slots.iter().any(|slot| slot.key.as_ref() == Some(key)) {
            UiIntentOccupancyPosture::InFlight
        } else {
            UiIntentOccupancyPosture::Idle
        }
    }
}

impl UiIntentOccupancyPlacement {
    pub(crate) const fn slots_inspected(&self) -> usize {
        self.slots_inspected
    }
}

impl UiIntentOccupancyPreparationFailure {
    const fn new(denial: UiIntentOccupancyReservationDenial, slots_inspected: usize) -> Self {
        Self {
            denial,
            slots_inspected,
        }
    }

    pub(crate) const fn denial(&self) -> UiIntentOccupancyReservationDenial {
        self.denial
    }

    pub(crate) const fn slots_inspected(&self) -> usize {
        self.slots_inspected
    }
}

impl UiIntentOccupancyObservation {
    pub(crate) const fn posture(&self) -> UiIntentOccupancyPosture {
        self.posture
    }
}

fn occupancy_key(
    scope: crate::declaration::UiIntentConcurrencyScope,
    declaration: &crate::declaration::UiCanonicalIntentDeclaration,
    definition: crate::capability::UiIntentId,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
) -> UiIntentOccupancyKey {
    match scope {
        crate::declaration::UiIntentConcurrencyScope::TargetRouteSingleFlight => {
            UiIntentOccupancyKey::TargetRoute {
                surface: target.surface(),
                binding: target.binding(),
                mounted_instance: target.mounted_instance(),
                declaration: declaration.identity().clone(),
            }
        }
        crate::declaration::UiIntentConcurrencyScope::DeclarationSingleFlight => {
            UiIntentOccupancyKey::Declaration(declaration.identity().clone())
        }
        crate::declaration::UiIntentConcurrencyScope::DefinitionSingleFlight => {
            UiIntentOccupancyKey::Definition(definition)
        }
        crate::declaration::UiIntentConcurrencyScope::ApplicationSingleFlight => {
            UiIntentOccupancyKey::Application
        }
    }
}
