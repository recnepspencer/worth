//! Runtime admission for existing companion registration and head-cell effects.
#[cfg(test)]
use super::guard_fault;
use super::{registry_guard, CompanionRegistrationState, PublicationCompanionRegistrationStop};
use crate::mvcc::publication::companion::RelationalPublicationCompanion;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::RelationalRuntimeOwnerBinding;
use std::sync::Arc;

#[path = "head_cell.rs"]
mod head_cell;
#[path = "retirement.rs"]
mod retirement;
pub use retirement::{
    PublicationCompanionRetirementDenial, RetiredPublicationCompanionRegistration,
};

/// A runtime-owned registration entry, separate from individual publications.
#[derive(Debug, Clone)]
pub struct PublicationCompanionRegistrationPort {
    runtime_instance_id: u64,
    owner: RelationalRuntimeOwnerBinding,
    pub(super) publication: RelationalRuntimePublicationBinding,
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
    pub(crate) fn new(
        runtime_instance_id: u64,
        owner: RelationalRuntimeOwnerBinding,
        publication: RelationalRuntimePublicationBinding,
    ) -> Self {
        Self {
            runtime_instance_id,
            owner,
            publication,
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
        let generation = self.publication.companion_registry().begin_required()?;
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
        let mut state = registry_guard::acquire(
            self.publication.companion_registry().state.try_write(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        #[cfg(test)]
        guard_fault::after_acquisition(guard_fault::Door::Remove);
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
    pub fn activate(
        self,
        participant: Arc<dyn RelationalPublicationCompanion>,
    ) -> Result<PublicationCompanionRegistration, PublicationCompanionRegistrationStop> {
        let _operation = self
            .port
            .owner
            .admit()
            .ok_or(PublicationCompanionRegistrationStop::OwnerUnavailable)?;
        let mut state = registry_guard::acquire(
            self.port.publication.companion_registry().state.try_write(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        #[cfg(test)]
        guard_fault::after_acquisition(guard_fault::Door::Activate);
        match &*state {
            CompanionRegistrationState::RequiredRebind { generation }
                if *generation == self.generation =>
            {
                let client = Arc::new(());
                *state = CompanionRegistrationState::RequiredActive {
                    generation: self.generation,
                    participant,
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
