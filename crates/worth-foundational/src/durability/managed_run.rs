//! Home-scoped identity of original managed work; no native execution authority.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ManagedRunObligationIdentity {
    home: HomeIdentity,
    acquire: NonZeroU64,
}
impl ManagedRunObligationIdentity {
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
impl identity_seal::Sealed for ManagedRunObligationIdentity {}
impl DurableIdentity for ManagedRunObligationIdentity {}
