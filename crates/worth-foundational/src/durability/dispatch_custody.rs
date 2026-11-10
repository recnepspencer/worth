//! Recorded external-dispatch attempt identity, never a live dispatch grant.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;

/// The attempt record's original position in this Home's dispatch journal.
///
/// The position is issued by the journal that durably records the attempt
/// before external contact, so it never depends on a process or runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DispatchAttemptIdentity {
    home: HomeIdentity,
    attempt: NonZeroU64,
}
impl DispatchAttemptIdentity {
    pub const fn from_recorded(home: HomeIdentity, attempt: NonZeroU64) -> Self {
        Self { home, attempt }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn attempt_position(self) -> NonZeroU64 {
        self.attempt
    }
}
impl identity_seal::Sealed for DispatchAttemptIdentity {}
impl DurableIdentity for DispatchAttemptIdentity {}
