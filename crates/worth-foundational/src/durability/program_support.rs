//! Recorded support meaning and obligation identity, never a live grant.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramSupportStatus {
    Active,
    Retiring,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgramSupportObligationKind {
    OriginalInterpretation,
    BranchAdoption,
}

/// The acquire record's original position in this Home's Query journal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProgramSupportObligationIdentity {
    home: HomeIdentity,
    acquire: NonZeroU64,
}
impl ProgramSupportObligationIdentity {
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
impl identity_seal::Sealed for ProgramSupportObligationIdentity {}
impl DurableIdentity for ProgramSupportObligationIdentity {}
