use std::sync::Arc;

use super::{
    PhysicalReadProtectionDenial, PhysicalReadProtectionObservation,
    PhysicalReadProtectionObserver, PhysicalReadProtectionPolicy, RootProtectionRegistry,
};
use crate::physical_runtime::{lifecycle::LifecycleState, RuntimeIdentity};

pub(in crate::physical_runtime) struct PhysicalReadProtectionOwner {
    registry: Arc<RootProtectionRegistry>,
}

impl PhysicalReadProtectionOwner {
    pub(in crate::physical_runtime) fn admit(
        policy: PhysicalReadProtectionPolicy,
        runtime: RuntimeIdentity,
        lifecycle: Arc<LifecycleState>,
    ) -> Result<Self, PhysicalReadProtectionDenial> {
        Ok(Self {
            registry: Arc::new(RootProtectionRegistry::admit(policy, runtime, lifecycle)?),
        })
    }

    pub(in crate::physical_runtime) fn registry(&self) -> Arc<RootProtectionRegistry> {
        Arc::clone(&self.registry)
    }

    pub(in crate::physical_runtime) fn observer(&self) -> PhysicalReadProtectionObserver {
        PhysicalReadProtectionObserver::new(self.registry())
    }

    pub(in crate::physical_runtime) fn close(self) -> PhysicalReadProtectionShutdown {
        self.registry.revoke();
        let observation = self.observer().snapshot();
        PhysicalReadProtectionShutdown {
            disposition: if observation.live_acquisitions() == 0 {
                PhysicalReadProtectionDisposition::Released
            } else {
                PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
            },
            observation,
        }
    }
}

impl Drop for PhysicalReadProtectionOwner {
    fn drop(&mut self) {
        self.registry.revoke();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalReadProtectionDisposition {
    Released,
    RetainedUntilReadersRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadProtectionShutdown {
    disposition: PhysicalReadProtectionDisposition,
    observation: PhysicalReadProtectionObservation,
}

impl PhysicalReadProtectionShutdown {
    pub const fn disposition(self) -> PhysicalReadProtectionDisposition {
        self.disposition
    }
    pub const fn observation(self) -> PhysicalReadProtectionObservation {
        self.observation
    }
}
