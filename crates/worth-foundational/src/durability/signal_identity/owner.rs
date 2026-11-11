use super::super::{identity_seal, DurableIdentity, HomeIdentity};

/// Stable owner slot inside a Home; reading it grants no attachment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignalOwnerSlotIdentity(u64);
impl SignalOwnerSlotIdentity {
    pub const fn from_recorded(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl identity_seal::Sealed for SignalOwnerSlotIdentity {}
impl DurableIdentity for SignalOwnerSlotIdentity {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalOwnerIdentity {
    home: HomeIdentity,
    slot: SignalOwnerSlotIdentity,
}
impl DurableSignalOwnerIdentity {
    pub const fn from_recorded(home: HomeIdentity, slot: SignalOwnerSlotIdentity) -> Self {
        Self { home, slot }
    }
    pub const fn home(self) -> HomeIdentity {
        self.home
    }
    pub const fn slot(self) -> SignalOwnerSlotIdentity {
        self.slot
    }
}
impl identity_seal::Sealed for DurableSignalOwnerIdentity {}
impl DurableIdentity for DurableSignalOwnerIdentity {}
