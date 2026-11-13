//! Descriptive identity of a cleanup obligation in its Home's Query history.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ManagedCleanupObligationIdentity {
    home: HomeIdentity,
    acquire: NonZeroU64,
}
impl ManagedCleanupObligationIdentity {
    pub const fn from_recorded(home: HomeIdentity, acquire: NonZeroU64) -> Self {
        Self { home, acquire }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn acquire_position(self) -> NonZeroU64 {
        self.acquire
    }
}
impl identity_seal::Sealed for ManagedCleanupObligationIdentity {}
impl DurableIdentity for ManagedCleanupObligationIdentity {}
