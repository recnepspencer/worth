use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct BridgeConditionalLoweringLease {
    live: AtomicBool,
}

impl BridgeConditionalLoweringLease {
    pub(super) fn issue() -> Self {
        Self {
            live: AtomicBool::new(true),
        }
    }

    pub(super) fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }

    pub(super) fn revoke_liveness(&self) {
        self.live.store(false, Ordering::Release);
    }
}

impl super::BridgeOwnedSignalRuntime {
    /// Graph identity alone cannot keep a retired or revoked installation live.
    /// The runtime's installed entry and its retained lease must agree before
    /// opening a source or executing against Signal.
    pub(super) fn require_live_installed_lowering(
        &self,
        lowering: &std::sync::Arc<super::BridgeInstalledConditionalLowering>,
    ) -> Result<(), super::BridgeConditionalDenial> {
        if lowering.bridge_runtime_key != self.bridge.signal_runtime_key
            || !lowering.lease.is_live()
            || !self
                .conditional_lowerings
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(super::contract::lowering_key(lowering))
                .is_some_and(|installed| std::sync::Arc::ptr_eq(installed, lowering))
        {
            return Err(super::BridgeConditionalDenial::new(
                super::BridgeConditionalDenialKind::StaleLowering,
                "conditional execution requires its exact live installed lowering",
            ));
        }
        Ok(())
    }
}
