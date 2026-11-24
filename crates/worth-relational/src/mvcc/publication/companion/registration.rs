use crate::publication::close_custody::RelationalPublicationCompanionCustody;
use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

use super::RelationalPublicationCompanion;

#[cfg(test)]
mod guard_fault;
#[cfg(test)]
mod poison_recovery;
pub(crate) mod port;
mod registry_guard;

#[derive(Debug)]
pub(crate) enum CompanionRegistrationState {
    Standalone,
    RequiredRebind {
        generation: u64,
    },
    RequiredActive {
        generation: u64,
        participant: Arc<dyn RelationalPublicationCompanion>,

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
        let state = registry_guard::acquire(
            self.state.read().map_err(std::sync::TryLockError::Poisoned),
            PublicationCompanionRegistrationStop::PublicationPending,
        )
        .expect(/* Proof: Blocking RwLock::read returns only a guard or PoisonError; the contention variant belongs exclusively to try_read. */ "blocking read has no contention stop");
        #[cfg(test)]
        guard_fault::after_acquisition(guard_fault::Door::Fork);
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

    pub(super) fn begin_required(&self) -> Result<u64, PublicationCompanionRegistrationStop> {
        let mut state = registry_guard::acquire(
            self.state.try_write(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        #[cfg(test)]
        guard_fault::after_acquisition(guard_fault::Door::Begin);
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
        Ok(generation)
    }

    pub(crate) fn enter(
        &self,
    ) -> Result<CompanionRegistrationEpoch<'_>, PublicationCompanionRegistrationStop> {
        let state = registry_guard::acquire(
            self.state.try_read(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        #[cfg(test)]
        guard_fault::after_acquisition(guard_fault::Door::Enter);
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
    pub(crate) fn close_custody(&self) -> RelationalPublicationCompanionCustody {
        match &*self.state {
            CompanionRegistrationState::Standalone
            | CompanionRegistrationState::RequiredRebind { .. } => {
                RelationalPublicationCompanionCustody::Released
            }
            CompanionRegistrationState::RequiredActive { generation, .. } => {
                if self._client.is_some() {
                    RelationalPublicationCompanionCustody::Client {
                        generation: *generation,
                    }
                } else {
                    RelationalPublicationCompanionCustody::Released
                }
            }
        }
    }
    /// Close observes actual client lifetime, independently of the required
    /// publication contract which continues to refuse writes until rebind.
    #[cfg(test)]
    pub(crate) fn has_live_close_client(&self) -> bool {
        self._client.is_some()
    }

    pub(crate) fn active(
        &self,
    ) -> Result<
        Option<(u64, &dyn RelationalPublicationCompanion)>,
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
                ..
            } if self._client.is_some() => Ok(Some((*generation, participant.as_ref()))),
            CompanionRegistrationState::RequiredActive { .. } => {
                Err(PublicationCompanionRegistrationStop::RebindRequired)
            }
        }
    }
}

pub use super::registration_stop::PublicationCompanionRegistrationStop;
