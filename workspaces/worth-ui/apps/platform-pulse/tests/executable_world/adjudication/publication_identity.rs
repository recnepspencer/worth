use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseApplicationGenerationObservation, PlatformPulseFirstFramePublished,
    PlatformPulseMountedFrameObservation, PlatformPulseReplacementPreserved,
    PlatformPulseReplacementPublished, PlatformPulseSourceSnapshotObservation,
};

use crate::external_observation::{NativeWindowIdentity, ProcessBoundNativeClientAreaObservation};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutablePublishedIdentity {
    source: PlatformPulseSourceSnapshotObservation,
    generation: PlatformPulseApplicationGenerationObservation,
    frame: PlatformPulseMountedFrameObservation,
    run: String,
    process_id: u32,
    window: NativeWindowIdentity,
}

struct ProductPublishedIdentity {
    source: PlatformPulseSourceSnapshotObservation,
    generation: PlatformPulseApplicationGenerationObservation,
    frame: PlatformPulseMountedFrameObservation,
}

impl ExecutablePublishedIdentity {
    pub(super) fn from_first_frame(
        published: PlatformPulseFirstFramePublished,
        run: &str,
        client: ProcessBoundNativeClientAreaObservation,
    ) -> Self {
        Self::from_parts(
            ProductPublishedIdentity {
                source: published.source(),
                generation: published.generation(),
                frame: published.frame(),
            },
            run,
            client,
        )
    }

    pub(super) fn from_replacement(
        published: PlatformPulseReplacementPublished,
        run: &str,
        client: ProcessBoundNativeClientAreaObservation,
    ) -> Self {
        Self::from_parts(
            ProductPublishedIdentity {
                source: published.source(),
                generation: published.active_generation(),
                frame: published.successor_frame(),
            },
            run,
            client,
        )
    }

    pub(super) fn from_preservation(
        preserved: PlatformPulseReplacementPreserved,
        run: &str,
        client: ProcessBoundNativeClientAreaObservation,
    ) -> Self {
        Self::from_parts(
            ProductPublishedIdentity {
                source: preserved.source(),
                generation: preserved.active_generation(),
                frame: preserved.active_frame(),
            },
            run,
            client,
        )
    }

    pub(crate) fn source(&self) -> PlatformPulseSourceSnapshotObservation {
        self.source
    }

    pub(crate) fn generation(&self) -> PlatformPulseApplicationGenerationObservation {
        self.generation
    }

    pub(crate) fn frame(&self) -> PlatformPulseMountedFrameObservation {
        self.frame
    }

    pub(crate) fn run(&self) -> &str {
        &self.run
    }

    pub(crate) fn process_id(&self) -> u32 {
        self.process_id
    }

    pub(crate) fn window(&self) -> NativeWindowIdentity {
        self.window
    }

    fn from_parts(
        product: ProductPublishedIdentity,
        run: &str,
        client: ProcessBoundNativeClientAreaObservation,
    ) -> Self {
        Self {
            source: product.source,
            generation: product.generation,
            frame: product.frame,
            run: run.to_owned(),
            process_id: client.process_id(),
            window: client.window(),
        }
    }
}
