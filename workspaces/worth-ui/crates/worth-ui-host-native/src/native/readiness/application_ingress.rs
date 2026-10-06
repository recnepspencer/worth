use super::{UiNativeReadinessRegistry, UiNativeReadyOwner, UiNativeWakeSender};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeApplicationWake;

/// Opaque level-triggered ingress for one application-owned readiness lane.
///
/// The application worker commits its semantic payload to its own bounded
/// channel before signalling this port. The host receives no payload or
/// subsystem meaning, only a wake for the exact registered owner.
#[derive(Clone)]
pub struct UiNativeApplicationReadinessPort {
    registry: UiNativeReadinessRegistry,
    owner: UiNativeReadyOwner,
    wake: UiNativeWakeSender,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationReadinessSignalDisposition {
    WakeRequested,
    Coalesced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeApplicationReadinessSignalDenial {
    OwnerClosed,
    EventLoopClosed,
}

impl UiNativeApplicationReadinessPort {
    pub(crate) fn new(
        registry: UiNativeReadinessRegistry,
        owner: UiNativeReadyOwner,
        wake: UiNativeWakeSender,
    ) -> Self {
        Self {
            registry,
            owner,
            wake,
        }
    }

    pub fn signal(
        &self,
    ) -> Result<
        UiNativeApplicationReadinessSignalDisposition,
        UiNativeApplicationReadinessSignalDenial,
    > {
        let queued = self
            .registry
            .signal_level(self.owner)
            .map_err(|()| UiNativeApplicationReadinessSignalDenial::OwnerClosed)?;
        if !queued {
            return Ok(UiNativeApplicationReadinessSignalDisposition::Coalesced);
        }
        if self.wake.send().is_err() {
            let _ = self.registry.cancel_level_signal(self.owner);
            return Err(UiNativeApplicationReadinessSignalDenial::EventLoopClosed);
        }
        Ok(UiNativeApplicationReadinessSignalDisposition::WakeRequested)
    }
}
