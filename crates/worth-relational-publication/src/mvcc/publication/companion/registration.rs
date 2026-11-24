use std::fmt::Debug;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};
use worth_relational_contracts::internal_port::publication::close_custody::RelationalPublicationCompanionCustody;

use super::RelationalPublicationCompanion;

#[doc(hidden)]
pub mod guard_fault;
pub mod readmitted_source;
pub use readmitted_source::install_readmitted_source_image;

pub mod registry_guard;
#[doc(hidden)]
pub mod retirement_participant;

#[derive(Debug)]
pub enum CompanionRegistrationState {
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
pub struct CompanionRegistry {
    pub state: RwLock<CompanionRegistrationState>,
    guard_fault: std::sync::atomic::AtomicU8,
}

impl CompanionRegistry {
    pub(crate) fn new() -> Self {
        Self {
            state: RwLock::new(CompanionRegistrationState::Standalone),
            guard_fault: Default::default(),
        }
    }

    pub fn fork(&self) -> Self {
        let state = registry_guard::acquire(
            self.state.read().map_err(std::sync::TryLockError::Poisoned),
            PublicationCompanionRegistrationStop::PublicationPending,
        )
        .expect(/* Proof: Blocking RwLock::read returns only a guard or PoisonError; the contention variant belongs exclusively to try_read. */ "blocking read has no contention stop");
        self.after_guard_acquisition(guard_fault::Door::Fork);
        let next = match &*state {
            CompanionRegistrationState::Standalone => CompanionRegistrationState::Standalone,
            CompanionRegistrationState::RequiredRebind { .. }
            | CompanionRegistrationState::RequiredActive { .. } => {
                CompanionRegistrationState::RequiredRebind { generation: 1 }
            }
        };
        Self {
            state: RwLock::new(next),
            guard_fault: Default::default(),
        }
    }

    pub fn begin_required(&self) -> Result<u64, PublicationCompanionRegistrationStop> {
        let mut state = registry_guard::acquire(
            self.state.try_write(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        self.after_guard_acquisition(guard_fault::Door::Begin);
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

    pub fn enter(
        &self,
    ) -> Result<CompanionRegistrationEpoch<'_>, PublicationCompanionRegistrationStop> {
        let state = registry_guard::acquire(
            self.state.try_read(),
            PublicationCompanionRegistrationStop::PublicationPending,
        )?;
        self.after_guard_acquisition(guard_fault::Door::Enter);
        let client = match &*state {
            CompanionRegistrationState::RequiredActive { client, .. } => client.upgrade(),
            _ => None,
        };
        Ok(CompanionRegistrationEpoch {
            state,
            _client: client,
        })
    }

    pub fn with_head_exclusion<R>(
        &self,
        inspect: impl FnOnce(&CompanionRegistrationState) -> R,
    ) -> Result<R, PublicationCompanionRegistrationStop> {
        let exclusion = registry_guard::acquire(
            self.state.try_write(),
            PublicationCompanionRegistrationStop::HeadCellPublicationContended,
        )?;
        self.after_guard_acquisition(guard_fault::Door::Head);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| inspect(&exclusion)));
        // Release normally before resuming the caller's unwind: publication
        // exclusion must not be poisoned by arbitrary derived installation.
        drop(exclusion);
        match result {
            Ok(value) => Ok(value),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
}

pub struct CompanionRegistrationEpoch<'a> {
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
    #[doc(hidden)]
    pub fn has_live_close_client_for_test(&self) -> bool {
        self._client.is_some()
    }

    #[doc(hidden)]
    pub fn rebind_generation_for_test(&self) -> Option<u64> {
        match &*self.state {
            CompanionRegistrationState::RequiredRebind { generation } => Some(*generation),
            _ => None,
        }
    }

    pub fn active(
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

pub use worth_relational_contracts::internal_port::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop;
