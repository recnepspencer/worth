//! A poisoned epoch keeps its typed state; only live contention is pending.
use super::PublicationCompanionRegistrationStop;
use std::sync::{TryLockError, TryLockResult};
pub(super) fn acquire<Guard>(
    attempt: TryLockResult<Guard>,
    contention: PublicationCompanionRegistrationStop,
) -> Result<Guard, PublicationCompanionRegistrationStop> {
    match attempt {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(poison)) => Ok(poison.into_inner()),
        Err(TryLockError::WouldBlock) => Err(contention),
    }
}
