//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn assert_no_live_handles_for_test(&self) {
        Self::assert_no_live_handles(self)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn enumerate_live_for_test(&self) -> Vec<WorthQueryRecoveryRegistrySlot> {
        Self::enumerate_live(self)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn is_live_for_test(&self, slot: WorthQueryRecoveryRegistrySlot) -> bool {
        Self::is_live(self, slot)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn mark_terminal_for_test(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
        terminal: WorthQueryRecoveryResourceTerminal,
    ) -> bool {
        Self::mark_terminal(self, slot, terminal)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn new_for_test() -> Self {
        Self::new()
    }
}

impl WorthQueryRecoveryMintClaim {
    #[doc(hidden)]
    pub fn new_for_test(
        provider_runtime_instance_id: u64,
        product_commit: CompositeCommitIdentity,
    ) -> Self {
        Self::new(provider_runtime_instance_id, product_commit)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn register_axis_probe_for_test(&self) -> WorthQueryRecoveryRegistrySlot {
        Self::register_axis_probe(self)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn register_once_for_test(
        &self,
        claim: WorthQueryRecoveryMintClaim,
    ) -> Result<WorthQueryRecoveryRegistrySlot, WorthQueryRecoveryMintAlreadyClaimed> {
        Self::register_once(self, claim)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn relinquish_for_test(&self, slot: WorthQueryRecoveryRegistrySlot) -> bool {
        Self::relinquish(self, slot)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn terminal_of_for_test(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
    ) -> Option<WorthQueryRecoveryResourceTerminal> {
        Self::terminal_of(self, slot)
    }
}

impl WorthQueryRecoveryRegistrySlot {
    #[doc(hidden)]
    pub fn as_u64_for_test(self) -> u64 {
        Self::as_u64(self)
    }
}

impl WorthQueryRecoveryHandleRegistry {
    #[doc(hidden)]
    pub fn clock_for_test(&self) -> &WorthQueryRuntimeClock {
        Self::clock(self)
    }
}
