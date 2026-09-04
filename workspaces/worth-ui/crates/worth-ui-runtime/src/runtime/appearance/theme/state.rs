use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

const UI_PREPARED_THEME_SWITCH_CAPACITY: usize = 4;

pub(crate) struct UiAppearanceThemeState {
    bindings:
        BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, super::UiActiveThemeBinding>,
    prepared: BTreeMap<u64, UiPreparedThemeReservation>,
    next_reservation: u64,
    owner_affinity: u64,
}

struct UiPreparedThemeReservation {
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    predecessor_generation: u64,
    owner_affinity: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiThemeInitialBindingDenial {
    SurfaceAlreadyBound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiThemeSwitchDenial {
    MissingActiveBinding,
    StaleBinding,
    WrongSurfaceCapability,
    WrongApplicationCapability,
    WrongOriginSession,
    BindingGenerationExhausted,
    PreparedSwitchCapacityExceeded,
    PreparedReservationExhausted,
    UnknownPreparedSwitch,
}

impl UiAppearanceThemeState {
    pub(crate) fn active_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = &super::UiActiveThemeBinding> {
        self.bindings.values()
    }

    pub(crate) fn active_binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&super::UiActiveThemeBinding> {
        self.bindings.get(&surface)
    }

    pub(crate) fn has_prepared_switches(&self) -> bool {
        !self.prepared.is_empty()
    }

    pub(crate) fn replace_carried_bindings(
        &mut self,
        bindings: impl IntoIterator<Item = super::UiActiveThemeBinding>,
    ) {
        self.bindings = bindings
            .into_iter()
            .map(|binding| (binding.surface(), binding))
            .collect();
    }

    pub(crate) fn install_initial(
        &mut self,
        capability: super::UiThemeCapabilityReceipt,
    ) -> Result<(), UiThemeInitialBindingDenial> {
        let surface = capability.surface();
        if self.bindings.contains_key(&surface) {
            return Err(UiThemeInitialBindingDenial::SurfaceAlreadyBound);
        }
        self.bindings.insert(
            surface,
            super::UiActiveThemeBinding {
                surface,
                binding_generation: 1,
                capability,
            },
        );
        Ok(())
    }

    pub(crate) fn prepare_generation_rebinding(
        &self,
        prepared: &super::UiPreparedThemeGenerationRebinding,
    ) -> Result<Box<[super::UiActiveThemeBinding]>, super::UiThemeCapabilityReceiptDenial> {
        if prepared.replacements().len() != self.bindings.len() {
            return Err(super::UiThemeCapabilityReceiptDenial::StaleBinding);
        }
        let mut successors = Vec::with_capacity(prepared.replacements().len());
        for replacement in prepared.replacements() {
            let current = self
                .bindings
                .get(&replacement.surface())
                .ok_or(super::UiThemeCapabilityReceiptDenial::StaleBinding)?;
            if current.binding_generation != replacement.predecessor_generation() {
                return Err(super::UiThemeCapabilityReceiptDenial::StaleBinding);
            }
            if current.capability.application() != prepared.predecessor() {
                return Err(super::UiThemeCapabilityReceiptDenial::GenerationMismatch);
            }
            if replacement.capability().application() != prepared.successor() {
                return Err(super::UiThemeCapabilityReceiptDenial::GenerationMismatch);
            }
            let binding_generation = current
                .binding_generation
                .checked_add(1)
                .ok_or(super::UiThemeCapabilityReceiptDenial::BindingGenerationExhausted)?;
            successors.push(super::UiActiveThemeBinding {
                surface: replacement.surface(),
                binding_generation,
                capability: replacement.capability().clone(),
            });
        }
        Ok(successors.into_boxed_slice())
    }

    #[cfg(test)]
    pub(crate) fn replace_for_test(&mut self, capability: super::UiThemeCapabilityReceipt) {
        let surface = capability.surface();
        let binding_generation = self.bindings.get(&surface).map_or(1, |binding| {
            binding
                .binding_generation
                .checked_add(1)
                .expect("test binding generation must not exhaust")
        });
        self.bindings.insert(
            surface,
            super::UiActiveThemeBinding {
                surface,
                binding_generation,
                capability,
            },
        );
    }

    #[cfg(test)]
    pub(crate) fn remove_for_test(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.bindings.remove(&surface).is_some()
    }

    pub(crate) fn prepare_theme_switch(
        &mut self,
        request: super::UiThemeSwitchRequest,
    ) -> Result<super::UiPreparedThemeSwitch, UiThemeSwitchDenial> {
        if request.capability.surface() != request.surface {
            return Err(UiThemeSwitchDenial::WrongSurfaceCapability);
        }
        if request.origin.session() != request.capability.application().session_identity() {
            return Err(UiThemeSwitchDenial::WrongOriginSession);
        }
        if request.origin.generation() != request.capability.application() {
            return Err(UiThemeSwitchDenial::WrongApplicationCapability);
        }
        let predecessor = self
            .bindings
            .get(&request.surface)
            .ok_or(UiThemeSwitchDenial::MissingActiveBinding)?;
        if predecessor.binding_generation != request.expected_binding_generation {
            return Err(UiThemeSwitchDenial::StaleBinding);
        }
        if predecessor.capability.application() != request.capability.application() {
            return Err(UiThemeSwitchDenial::WrongApplicationCapability);
        }
        let binding_generation = predecessor
            .binding_generation
            .checked_add(1)
            .ok_or(UiThemeSwitchDenial::BindingGenerationExhausted)?;
        if self.prepared.len() >= UI_PREPARED_THEME_SWITCH_CAPACITY {
            return Err(UiThemeSwitchDenial::PreparedSwitchCapacityExceeded);
        }
        let reservation = self
            .next_reservation
            .checked_add(1)
            .ok_or(UiThemeSwitchDenial::PreparedReservationExhausted)?;
        self.next_reservation = reservation;
        self.prepared.insert(
            reservation,
            UiPreparedThemeReservation {
                surface: request.surface,
                application: request.capability.application().clone(),
                predecessor_generation: predecessor.binding_generation,
                owner_affinity: self.owner_affinity,
            },
        );
        Ok(super::UiPreparedThemeSwitch {
            reservation,
            predecessor_generation: predecessor.binding_generation,
            successor: super::UiActiveThemeBinding {
                surface: request.surface,
                binding_generation,
                capability: request.capability,
            },
            origin: request.origin,
            owner_affinity: self.owner_affinity,
        })
    }

    pub(crate) fn commit_published_switch(
        &mut self,
        prepared: super::UiPreparedThemeSwitch,
    ) -> Result<(), UiThemeSwitchDenial> {
        let Some(reservation) = self.prepared.get(&prepared.reservation) else {
            return Err(UiThemeSwitchDenial::UnknownPreparedSwitch);
        };
        if reservation.surface != prepared.successor.surface
            || reservation.application != *prepared.successor.capability.application()
            || reservation.predecessor_generation != prepared.predecessor_generation
            || reservation.owner_affinity != prepared.owner_affinity
        {
            return Err(UiThemeSwitchDenial::UnknownPreparedSwitch);
        }
        let current = self
            .bindings
            .get(&prepared.successor.surface)
            .ok_or(UiThemeSwitchDenial::MissingActiveBinding)?;
        if current.binding_generation != prepared.predecessor_generation {
            return Err(UiThemeSwitchDenial::StaleBinding);
        }
        let committed_surface = prepared.successor.surface;
        let committed_application = prepared.successor.capability.application().clone();
        let committed_predecessor = prepared.predecessor_generation;
        self.prepared.retain(|_, competing| {
            competing.surface != committed_surface
                || competing.application != committed_application
                || competing.predecessor_generation != committed_predecessor
        });
        self.bindings
            .insert(prepared.successor.surface, prepared.successor);
        Ok(())
    }

    pub(crate) fn cancel_prepared_switch(
        &mut self,
        prepared: super::UiPreparedThemeSwitch,
    ) -> Result<(), UiThemeSwitchDenial> {
        let Some(reservation) = self.prepared.get(&prepared.reservation) else {
            return Err(UiThemeSwitchDenial::UnknownPreparedSwitch);
        };
        if reservation.surface != prepared.successor.surface
            || reservation.application != *prepared.successor.capability.application()
            || reservation.predecessor_generation != prepared.predecessor_generation
            || reservation.owner_affinity != prepared.owner_affinity
        {
            return Err(UiThemeSwitchDenial::UnknownPreparedSwitch);
        }
        self.prepared.remove(&prepared.reservation);
        Ok(())
    }

    pub(crate) fn prepared_switch_count(&self) -> usize {
        self.prepared.len()
    }
}

impl Default for UiAppearanceThemeState {
    fn default() -> Self {
        static NEXT_OWNER_AFFINITY: AtomicU64 = AtomicU64::new(1);
        let owner_affinity = NEXT_OWNER_AFFINITY.fetch_add(1, Ordering::Relaxed);
        assert!(owner_affinity != 0, "theme owner affinity exhausted");
        Self {
            bindings: BTreeMap::new(),
            prepared: BTreeMap::new(),
            next_reservation: 0,
            owner_affinity,
        }
    }
}
