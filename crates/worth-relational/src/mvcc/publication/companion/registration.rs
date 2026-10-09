use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

use crate::runtime::{
    PositionedRelationalSnapshot, RelationalRuntime, RelationalRuntimeOwnerBinding,
    RelationalRuntimePublicationBinding,
};

use super::{CompanionBranchCell, CompanionPreflightBudget, RelationalPublicationCompanion};

#[derive(Debug)]
pub(crate) enum CompanionRegistrationState {
    Standalone,
    RequiredRebind {
        generation: u64,
    },
    RequiredActive {
        generation: u64,
        participant: Arc<dyn RelationalPublicationCompanion>,
        budget: CompanionPreflightBudget,
        client: Weak<()>,
    },
}

#[derive(Debug)]
pub(crate) struct CompanionRegistry {
    state: RwLock<CompanionRegistrationState>,
}

impl CompanionRegistry {
    pub(crate) fn new() -> Self {
        Self {
            state: RwLock::new(CompanionRegistrationState::Standalone),
        }
    }

    pub(crate) fn fork(&self) -> Self {
        let state = self.state.read().unwrap_or_else(|p| p.into_inner());
        let next = match &*state {
            CompanionRegistrationState::Standalone => CompanionRegistrationState::Standalone,
            CompanionRegistrationState::RequiredRebind { .. }
            | CompanionRegistrationState::RequiredActive { .. } => {
                CompanionRegistrationState::RequiredRebind { generation: 1 }
            }
        };
        Self {
            state: RwLock::new(next),
        }
    }

    pub(crate) fn enter(
        &self,
    ) -> Result<CompanionRegistrationEpoch<'_>, PublicationCompanionRegistrationStop> {
        let state = self
            .state
            .try_read()
            .map_err(|_| PublicationCompanionRegistrationStop::PublicationPending)?;
        let client = match &*state {
            CompanionRegistrationState::RequiredActive { client, .. } => client.upgrade(),
            _ => None,
        };
        Ok(CompanionRegistrationEpoch {
            state,
            _client: client,
        })
    }
}

pub(crate) struct CompanionRegistrationEpoch<'a> {
    state: RwLockReadGuard<'a, CompanionRegistrationState>,
    _client: Option<Arc<()>>,
}

impl CompanionRegistrationEpoch<'_> {
    pub(crate) fn active(
        &self,
    ) -> Result<
        Option<(
            u64,
            &dyn RelationalPublicationCompanion,
            CompanionPreflightBudget,
        )>,
        PublicationCompanionRegistrationStop,
    > {
        match &*self.state {
            CompanionRegistrationState::Standalone => Ok(None),
            CompanionRegistrationState::RequiredRebind { .. } => {
                Err(PublicationCompanionRegistrationStop::RebindRequired)
            }
            CompanionRegistrationState::RequiredActive {
                generation,
                participant,
                budget,
                ..
            } if self._client.is_some() => Ok(Some((*generation, participant.as_ref(), *budget))),
            CompanionRegistrationState::RequiredActive { .. } => {
                Err(PublicationCompanionRegistrationStop::RebindRequired)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationCompanionRegistrationStop {
    OwnerUnavailable,
    PublicationPending,
    RebindRequired,
    Superseded,
    IdentityExhausted,
    ForeignSnapshot,
}

/// A runtime-owned registration entry, separate from individual publications.
#[derive(Debug, Clone)]
pub struct PublicationCompanionRegistrationPort {
    runtime_instance_id: u64,
    owner: RelationalRuntimeOwnerBinding,
    publication: RelationalRuntimePublicationBinding,
}

#[derive(Debug)]
pub struct PendingCompanionRegistration {
    port: PublicationCompanionRegistrationPort,
    generation: u64,
}

#[derive(Debug, Clone)]
pub struct PublicationCompanionRegistration {
    port: PublicationCompanionRegistrationPort,
    generation: u64,
    _client: Arc<()>,
}

impl PublicationCompanionRegistrationPort {
    pub(crate) fn new(runtime: &RelationalRuntime) -> Self {
        Self {
            runtime_instance_id: runtime.runtime_instance_id(),
            owner: runtime.owner_binding(),
            publication: runtime.publication_binding(),
        }
    }

    /// Enter required posture before Query begins building branch cells.
    pub fn begin_required_registration(
        &self,
    ) -> Result<PendingCompanionRegistration, PublicationCompanionRegistrationStop> {
        let _operation = self
            .owner
            .admit()
            .ok_or(PublicationCompanionRegistrationStop::OwnerUnavailable)?;
        let registry = self.publication.companion_registry();
        let mut state = registry
            .state
            .try_write()
            .map_err(|_| PublicationCompanionRegistrationStop::PublicationPending)?;
        let generation = match &*state {
            CompanionRegistrationState::Standalone => 1,
            CompanionRegistrationState::RequiredRebind { generation }
            | CompanionRegistrationState::RequiredActive { generation, .. } => generation
                .checked_add(1)
                .ok_or(PublicationCompanionRegistrationStop::IdentityExhausted)?,
        };
        let retired = std::mem::replace(
            &mut *state,
            CompanionRegistrationState::RequiredRebind { generation },
        );
        drop(state);
        drop(retired);
        Ok(PendingCompanionRegistration {
            port: self.clone(),
            generation,
        })
    }

    /// Removal retains the required posture so a direct writer cannot bypass Query.
    pub fn remove_required(
        &self,
        registration: &PublicationCompanionRegistration,
    ) -> Result<(), PublicationCompanionRegistrationStop> {
        self.require_local(registration)?;
        let _operation = self
            .owner
            .admit()
            .ok_or(PublicationCompanionRegistrationStop::OwnerUnavailable)?;
        let mut state = self
            .publication
            .companion_registry()
            .state
            .try_write()
            .map_err(|_| PublicationCompanionRegistrationStop::PublicationPending)?;
        match &*state {
            CompanionRegistrationState::RequiredActive { generation, .. }
                if *generation == registration.generation =>
            {
                let generation = *generation;
                let retired = std::mem::replace(
                    &mut *state,
                    CompanionRegistrationState::RequiredRebind { generation },
                );
                drop(state);
                drop(retired);
                Ok(())
            }
            _ => Err(PublicationCompanionRegistrationStop::Superseded),
        }
    }

    fn require_local(
        &self,
        registration: &PublicationCompanionRegistration,
    ) -> Result<(), PublicationCompanionRegistrationStop> {
        if self.runtime_instance_id != registration.port.runtime_instance_id
            || !self
                .publication
                .belongs_to_same_owner(&registration.port.publication)
        {
            return Err(PublicationCompanionRegistrationStop::Superseded);
        }
        Ok(())
    }
}

impl PendingCompanionRegistration {
    pub fn mint_branch_cell<T: Send + Sync + 'static>(
        &self,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, PublicationCompanionRegistrationStop> {
        self.port.mint_cell(self.generation, selected, initial)
    }

    pub fn activate(
        self,
        participant: Arc<dyn RelationalPublicationCompanion>,
        budget: CompanionPreflightBudget,
    ) -> Result<PublicationCompanionRegistration, PublicationCompanionRegistrationStop> {
        let _operation = self
            .port
            .owner
            .admit()
            .ok_or(PublicationCompanionRegistrationStop::OwnerUnavailable)?;
        let mut state = self
            .port
            .publication
            .companion_registry()
            .state
            .try_write()
            .map_err(|_| PublicationCompanionRegistrationStop::PublicationPending)?;
        match &*state {
            CompanionRegistrationState::RequiredRebind { generation }
                if *generation == self.generation =>
            {
                let client = Arc::new(());
                *state = CompanionRegistrationState::RequiredActive {
                    generation: self.generation,
                    participant,
                    budget,
                    client: Arc::downgrade(&client),
                };
                Ok(PublicationCompanionRegistration {
                    port: self.port.clone(),
                    generation: self.generation,
                    _client: client,
                })
            }
            _ => Err(PublicationCompanionRegistrationStop::Superseded),
        }
    }
}

impl PublicationCompanionRegistration {
    pub fn mint_branch_cell<T: Send + Sync + 'static>(
        &self,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, PublicationCompanionRegistrationStop> {
        self.port.mint_cell(self.generation, selected, initial)
    }
}

impl PublicationCompanionRegistrationPort {
    fn mint_cell<T: Send + Sync + 'static>(
        &self,
        generation: u64,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, PublicationCompanionRegistrationStop> {
        let _operation = self
            .owner
            .admit()
            .ok_or(PublicationCompanionRegistrationStop::OwnerUnavailable)?;
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(PublicationCompanionRegistrationStop::ForeignSnapshot);
        }
        let epoch = self.publication.companion_registry().enter()?;
        match &*epoch.state {
            CompanionRegistrationState::RequiredRebind { generation: active }
            | CompanionRegistrationState::RequiredActive {
                generation: active, ..
            } if *active == generation => {
                Ok(CompanionBranchCell::new(selected, generation, initial))
            }
            _ => Err(PublicationCompanionRegistrationStop::Superseded),
        }
    }
}

impl RelationalRuntime {
    pub fn publication_companion_port(&self) -> PublicationCompanionRegistrationPort {
        PublicationCompanionRegistrationPort::new(self)
    }
}
