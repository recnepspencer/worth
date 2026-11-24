//! A poisoned epoch keeps its typed state; only live contention is pending.
use std::sync::{TryLockError, TryLockResult};
use worth_relational_contracts::internal_port::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop;
pub fn acquire<Guard>(
    attempt: TryLockResult<Guard>,
    contention: PublicationCompanionRegistrationStop,
) -> Result<Guard, PublicationCompanionRegistrationStop> {
    match attempt {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(poison)) => Ok(poison.into_inner()),
        Err(TryLockError::WouldBlock) => Err(contention),
    }
}
