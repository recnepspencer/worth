//! Stable recovery-obligation identity and disposition DATA, never a handle grant.
use super::{identity_seal, DurableIdentity, HomeIdentity};
use std::num::NonZeroU64;

/// The original acknowledged claim position in this Home's dispatch journal.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct DurableRecoveryClaimIdentity {
    home: HomeIdentity,
    position: NonZeroU64,
}
impl DurableRecoveryClaimIdentity {
    pub const fn from_recorded(home: HomeIdentity, position: NonZeroU64) -> Self {
        Self { home, position }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn position(self) -> NonZeroU64 {
        self.position
    }
}
impl identity_seal::Sealed for DurableRecoveryClaimIdentity {}
impl DurableIdentity for DurableRecoveryClaimIdentity {}

/// The existing recovery lifecycle's terminal meaning, shared by live and recorded owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RecoveryClaimDisposition {
    Consumed,
    Completed,
    Expired,
    Disposed,
    ForceTerminated,
    Relinquished,
}
impl worth_proof::TerminalState for RecoveryClaimDisposition {
    fn label(&self) -> &'static str {
        match self {
            Self::Consumed => "consumed",
            Self::Completed => "completed",
            Self::Expired => "expired",
            Self::Disposed => "disposed",
            Self::ForceTerminated => "force-terminated",
            Self::Relinquished => "relinquished",
        }
    }
}
