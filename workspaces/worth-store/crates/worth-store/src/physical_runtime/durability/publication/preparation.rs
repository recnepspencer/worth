use std::sync::{Arc, Mutex, Weak};

use worth_store_physical_format::DurablePhysicalRootManifest;

use super::PhysicalRootPublicationIdentity;

pub(in crate::physical_runtime) struct PhysicalRootPublicationTransitionOwner {
    state: Arc<Mutex<PhysicalRootPublicationTransitionState>>,
    runtime: Weak<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
}

pub(in crate::physical_runtime) struct PhysicalRootPublicationTransition {
    state: Arc<Mutex<PhysicalRootPublicationTransitionState>>,
    identity: PhysicalRootPublicationIdentity,
    source_root: DurablePhysicalRootManifest,
    active: bool,
    effect_started: bool,
    runtime: Weak<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootPublicationTransitionDenial {
    CurrentRootMismatch,
    TransitionActive,
    InspectionRequired,
    ReclaimFenced,
}

enum PhysicalRootPublicationTransitionState {
    Available,
    Active(PhysicalRootPublicationIdentity),
    InspectionRequired,
}

impl PhysicalRootPublicationTransitionOwner {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(
                PhysicalRootPublicationTransitionState::Available,
            )),
            runtime: Arc::downgrade(runtime),
        }
    }

    pub(in crate::physical_runtime) fn begin(
        &self,
        identity: PhysicalRootPublicationIdentity,
        current_root: &DurablePhysicalRootManifest,
        source_root: DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionDenial> {
        if source_root != *current_root || source_root.generation() != identity.source_generation()
        {
            return Err(PhysicalRootPublicationTransitionDenial::CurrentRootMismatch);
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match *state {
            PhysicalRootPublicationTransitionState::Available => {
                *state = PhysicalRootPublicationTransitionState::Active(identity);
            }
            PhysicalRootPublicationTransitionState::Active(_) => {
                return Err(PhysicalRootPublicationTransitionDenial::TransitionActive);
            }
            PhysicalRootPublicationTransitionState::InspectionRequired => {
                return Err(PhysicalRootPublicationTransitionDenial::InspectionRequired);
            }
        }
        drop(state);
        Ok(PhysicalRootPublicationTransition {
            state: Arc::clone(&self.state),
            identity,
            source_root,
            active: true,
            effect_started: false,
            runtime: self.runtime.clone(),
        })
    }
}

impl PhysicalRootPublicationTransition {
    pub(in crate::physical_runtime) const fn identity(&self) -> PhysicalRootPublicationIdentity {
        self.identity
    }

    pub(in crate::physical_runtime) const fn source_root(&self) -> &DurablePhysicalRootManifest {
        &self.source_root
    }

    pub(in crate::physical_runtime) fn mark_effect_started(&mut self) {
        self.effect_started = true;
    }

    pub(in crate::physical_runtime) fn prove_no_effect(&mut self) {
        self.effect_started = false;
    }

    pub(in crate::physical_runtime) fn require_inspection(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(
            *state,
            PhysicalRootPublicationTransitionState::Active(active) if active == self.identity
        ) {
            *state = PhysicalRootPublicationTransitionState::InspectionRequired;
        }
        self.active = false;
    }

    pub(in crate::physical_runtime) fn release(mut self) {
        self.release_available();
    }

    fn release_available(&mut self) {
        if !self.active {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(
            *state,
            PhysicalRootPublicationTransitionState::Active(active) if active == self.identity
        ) {
            *state = PhysicalRootPublicationTransitionState::Available;
        }
        self.active = false;
    }

    fn abandon_if_effect_started(&mut self) {
        if !self.active || !self.effect_started {
            self.release_available();
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if matches!(
            *state,
            PhysicalRootPublicationTransitionState::Active(active) if active == self.identity
        ) {
            *state = PhysicalRootPublicationTransitionState::InspectionRequired;
        }
        self.active = false;
        drop(state);
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.health.revoke();
        }
    }
}

impl Drop for PhysicalRootPublicationTransition {
    fn drop(&mut self) {
        self.abandon_if_effect_started();
    }
}
