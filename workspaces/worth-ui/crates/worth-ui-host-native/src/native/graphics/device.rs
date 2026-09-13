use std::sync::Arc;

use super::{UiNativeBackendDeviceGenerationMechanics, UiNativeBackendDeviceMechanics};

#[cfg(test)]
#[path = "device_pipeline_tests.rs"]
mod pipeline_tests;

pub(crate) struct UiNativeDeviceState {
    pub(crate) mechanics: UiNativeBackendDeviceMechanics,
    pub(in crate::native::graphics) generation: std::sync::Arc<UiNativeDeviceGeneration>,
}

pub(crate) struct UiNativeDeviceGeneration {
    pub(crate) identity: u64,
    pub(crate) mechanics: UiNativeBackendDeviceGenerationMechanics,
    presentation_pipelines:
        std::sync::OnceLock<crate::native::presentation::UiNativePresentationPipelines>,
}

pub(crate) struct UiNativeDeviceOwners {
    pub(crate) adapter: crate::native::UiNativeResourceOwner,
    pub(crate) device: crate::native::UiNativeResourceOwner,
    pub(crate) queue: crate::native::UiNativeResourceOwner,
}

pub(crate) struct UiNativeOwnedDevice {
    state: Box<UiNativeDeviceState>,
    owners: UiNativeDeviceOwners,
    retired: Vec<UiNativeRetiredDeviceGeneration>,
}

struct UiNativeRetiredDeviceGeneration {
    generation: Arc<UiNativeDeviceGeneration>,
    device: crate::native::UiNativeResourceOwner,
    queue: crate::native::UiNativeResourceOwner,
}

impl UiNativeDeviceGeneration {
    pub(in crate::native) fn new(
        identity: u64,
        mechanics: UiNativeBackendDeviceGenerationMechanics,
    ) -> Self {
        Self {
            identity,
            mechanics,
            presentation_pipelines: std::sync::OnceLock::new(),
        }
    }

    pub(crate) fn presentation_pipelines(
        &self,
    ) -> &crate::native::presentation::UiNativePresentationPipelines {
        self.presentation_pipelines
            .get_or_init(|| crate::native::presentation::presentation_pipelines(self.device()))
    }
}

impl UiNativeOwnedDevice {
    pub(crate) fn new(state: UiNativeDeviceState, owners: UiNativeDeviceOwners) -> Self {
        Self {
            state: Box::new(state),
            owners,
            retired: Vec::new(),
        }
    }

    pub(crate) fn state(&self) -> &UiNativeDeviceState {
        &self.state
    }

    pub(crate) fn replace_generation(
        &mut self,
        generation: Arc<UiNativeDeviceGeneration>,
        device_owner: crate::native::UiNativeResourceOwner,
        queue_owner: crate::native::UiNativeResourceOwner,
    ) {
        let predecessor_generation = std::mem::replace(&mut self.state.generation, generation);
        let predecessor_device = std::mem::replace(&mut self.owners.device, device_owner);
        let predecessor_queue = std::mem::replace(&mut self.owners.queue, queue_owner);
        self.retired.push(UiNativeRetiredDeviceGeneration {
            generation: predecessor_generation,
            device: predecessor_device,
            queue: predecessor_queue,
        });
    }

    pub(crate) fn collect_settled(
        &mut self,
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), ()> {
        let retired = std::mem::take(&mut self.retired);
        for generation in retired {
            if Arc::strong_count(&generation.generation) == 1 {
                drop(generation.generation);
                registry.release(generation.device)?;
                registry.release(generation.queue)?;
            } else {
                self.retired.push(generation);
            }
        }
        Ok(())
    }

    pub(crate) fn close(
        mut self,
        registry: &mut crate::native::UiNativeResourceRegistry,
    ) -> Result<(), Self> {
        if self.collect_settled(registry).is_err()
            || self
                .retired
                .iter()
                .any(|retired| Arc::strong_count(&retired.generation) != 1)
        {
            return Err(self);
        }
        let UiNativeDeviceOwners {
            adapter,
            device,
            queue,
        } = self.owners;
        drop(self.state);
        registry
            .release_all([adapter, device, queue])
            .expect("device owners remain exact");
        Ok(())
    }

    pub(crate) fn can_close(&self) -> bool {
        self.retired
            .iter()
            .all(|retired| Arc::strong_count(&retired.generation) == 1)
    }
}
