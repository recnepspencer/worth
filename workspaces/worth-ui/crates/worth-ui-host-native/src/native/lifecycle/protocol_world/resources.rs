use crate::native::readiness::{signal_committed, UiNativeReadinessRegistry, UiNativeReadyOwner};
use crate::native::{UiNativeResourceClass, UiNativeResourceOwner, UiNativeResourceRegistry};

use super::schema::UiNativeLifecycleProtocolSchedule;

pub(in crate::native::lifecycle) struct UiProtocolResources {
    registry: UiNativeResourceRegistry,
    queued_readiness: Option<UiProtocolQueuedReadiness>,
    prepared_upload: Option<UiNativeResourceOwner>,
    surface: Option<UiNativeResourceOwner>,
    device: Option<UiNativeResourceOwner>,
    queue: Option<UiNativeResourceOwner>,
    presentation: Option<UiNativeResourceOwner>,
    readback: Option<UiNativeResourceOwner>,
}

struct UiProtocolQueuedReadiness {
    registry: UiNativeReadinessRegistry,
    owner: UiNativeReadyOwner,
}

impl UiProtocolResources {
    pub(in crate::native::lifecycle) fn new(schedule: UiNativeLifecycleProtocolSchedule) -> Self {
        let mut registry = UiNativeResourceRegistry::new();
        Self {
            queued_readiness: schedule
                .queued_readiness()
                .then(UiProtocolQueuedReadiness::queued)
                .transpose()
                .expect("protocol readiness capacity"),
            prepared_upload: register_if(
                &mut registry,
                schedule.close_point()
                    == Some(super::schema::UiNativeProtocolClosePoint::PreparedUpload),
                UiNativeResourceClass::AtlasStagingBuffer,
            ),
            surface: Some(register(&mut registry, UiNativeResourceClass::Surface)),
            device: Some(register(&mut registry, UiNativeResourceClass::Device)),
            queue: Some(register(&mut registry, UiNativeResourceClass::Queue)),
            presentation: Some(register(
                &mut registry,
                UiNativeResourceClass::PendingPresentation,
            )),
            readback: None,
            registry,
        }
    }

    pub(in crate::native::lifecycle) fn current(&self) -> crate::native::UiNativeResourceCensus {
        self.registry.current()
    }

    pub(in crate::native::lifecycle) fn peak(&self) -> crate::native::UiNativeResourceCensus {
        self.registry.peak()
    }

    pub(in crate::native::lifecycle) fn finish_queued_work(&mut self) {
        if let Some(queued) = self.queued_readiness.take() {
            let _ = queued.registry.take(queued.owner);
            let closed = queued.registry.close();
            debug_assert_eq!(closed, 1);
        }
    }

    pub(in crate::native::lifecycle) fn finish_presentation(&mut self) {
        release(&mut self.registry, &mut self.presentation);
        release(&mut self.registry, &mut self.readback);
    }

    pub(in crate::native::lifecycle) fn abandon_presentation(&mut self) {
        release(&mut self.registry, &mut self.presentation);
    }

    pub(in crate::native::lifecycle) fn begin_readback(&mut self) {
        if self.readback.is_none() {
            self.readback = Some(register(
                &mut self.registry,
                UiNativeResourceClass::ReadbackBuffer,
            ));
        }
    }

    pub(in crate::native::lifecycle) fn settle_readback(&mut self) {
        release(&mut self.registry, &mut self.readback);
    }

    pub(in crate::native::lifecycle) fn replace_surface(&mut self) {
        let successor = register(&mut self.registry, UiNativeResourceClass::Surface);
        release(&mut self.registry, &mut self.surface);
        self.surface = Some(successor);
    }

    pub(in crate::native::lifecycle) fn replace_device_and_queue(&mut self) {
        let successor_device = register(&mut self.registry, UiNativeResourceClass::Device);
        let successor_queue = register(&mut self.registry, UiNativeResourceClass::Queue);
        release(&mut self.registry, &mut self.device);
        release(&mut self.registry, &mut self.queue);
        self.device = Some(successor_device);
        self.queue = Some(successor_queue);
    }

    pub(in crate::native::lifecycle) fn settle_external(&mut self) -> bool {
        self.finish_presentation();
        true
    }

    pub(in crate::native::lifecycle) fn release_all(&mut self) {
        if let Some(queued) = self.queued_readiness.take() {
            let closed = queued.registry.close();
            debug_assert_eq!(closed, 1);
        }
        release(&mut self.registry, &mut self.prepared_upload);
        release(&mut self.registry, &mut self.presentation);
        release(&mut self.registry, &mut self.readback);
        release(&mut self.registry, &mut self.surface);
        release(&mut self.registry, &mut self.device);
        release(&mut self.registry, &mut self.queue);
    }

    pub(in crate::native::lifecycle) fn queued_readiness_count(&self) -> usize {
        usize::from(self.queued_readiness.is_some())
    }
}

impl UiProtocolQueuedReadiness {
    fn queued() -> Result<Self, ()> {
        let registry = UiNativeReadinessRegistry::new();
        let owner = registry.register()?;
        registry.commit_latest(owner, 1_000, [160, 96])?;
        let mut redraw_requests = 0;
        signal_committed(&registry, owner, || redraw_requests += 1)?;
        (redraw_requests == 1)
            .then_some(Self { registry, owner })
            .ok_or(())
    }
}

fn register(
    registry: &mut UiNativeResourceRegistry,
    class: UiNativeResourceClass,
) -> UiNativeResourceOwner {
    registry
        .register(class)
        .expect("protocol world resource capacity")
}

fn register_if(
    registry: &mut UiNativeResourceRegistry,
    required: bool,
    class: UiNativeResourceClass,
) -> Option<UiNativeResourceOwner> {
    required.then(|| register(registry, class))
}

fn release(registry: &mut UiNativeResourceRegistry, owner: &mut Option<UiNativeResourceOwner>) {
    if let Some(owner) = owner.take() {
        registry
            .release(owner)
            .expect("protocol resource owner remains exact until release");
    }
}
