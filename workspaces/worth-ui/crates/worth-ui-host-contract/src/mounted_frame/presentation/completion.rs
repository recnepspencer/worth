use std::rc::Rc;

/// The runtime-issued work a host acknowledges when it reports physical completion.
///
/// Captured when the runtime hands one surface's work to the host, either as the
/// consumption view itself or as the completion token issued from it. Only this
/// module can build it, so a completion always names the exact attempt, surface
/// binding, frame, and content the runtime issued.
#[derive(Clone)]
pub(in crate::mounted_frame) struct UiMountedIssuedSurfaceWork {
    authority: Rc<()>,
    attempt: crate::UiMountedPresentationAttemptIdentity,
    requirement: crate::UiMountedSurfaceBindingRequirement,
    frame: crate::UiMountedFrameIdentity,
    content: crate::UiMountedContentGeneration,
}

/// Host acknowledgement that one surface's frame physically completed.
///
/// This is the only evidence that a frame is on screen. It has no public
/// constructor: a host mints it by acknowledging the consumption view or the
/// completion token the runtime issued for that surface. The runtime admits it
/// only when it carries the runtime's own seal and the identities it issued.
pub struct UiMountedSurfacePresentationCompletion {
    work: UiMountedIssuedSurfaceWork,
    mode: crate::UiHostSurfacePresentationMode,
    epoch: crate::UiHostPresentationEpoch,
    effects: super::UiMountedCompletedEffects,
    cost: crate::UiHostPresentationCostReport,
}

impl UiMountedIssuedSurfaceWork {
    pub(in crate::mounted_frame) fn new(
        authority: Rc<()>,
        attempt: crate::UiMountedPresentationAttemptIdentity,
        requirement: crate::UiMountedSurfaceBindingRequirement,
        frame: crate::UiMountedFrameIdentity,
        content: crate::UiMountedContentGeneration,
    ) -> Self {
        Self {
            authority,
            attempt,
            requirement,
            frame,
            content,
        }
    }

    pub(in crate::mounted_frame) fn issued_by_runtime(&self, seal: &Rc<()>) -> bool {
        Rc::ptr_eq(&self.authority, seal)
    }

    pub(in crate::mounted_frame) fn acknowledge_presented(
        self,
        mode: crate::UiHostSurfacePresentationMode,
        epoch: crate::UiHostPresentationEpoch,
        effects: super::UiMountedCompletedEffects,
        cost: crate::UiHostPresentationCostReport,
    ) -> UiMountedSurfacePresentationCompletion {
        UiMountedSurfacePresentationCompletion {
            work: self,
            mode,
            epoch,
            effects,
            cost,
        }
    }
}

impl UiMountedSurfacePresentationCompletion {
    /// Whether this acknowledgement answers work the holder of `seal` issued.
    #[doc(hidden)]
    pub fn issued_by_runtime(&self, seal: &Rc<()>) -> bool {
        self.work.issued_by_runtime(seal)
    }

    pub fn attempt(&self) -> crate::UiMountedPresentationAttemptIdentity {
        self.work.attempt
    }

    pub fn requirement(&self) -> crate::UiMountedSurfaceBindingRequirement {
        self.work.requirement
    }

    pub fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.work.frame
    }

    pub fn content_generation(&self) -> crate::UiMountedContentGeneration {
        self.work.content
    }

    pub fn mode(&self) -> crate::UiHostSurfacePresentationMode {
        self.mode
    }

    pub fn epoch(&self) -> crate::UiHostPresentationEpoch {
        self.epoch
    }

    pub fn effects(&self) -> &super::UiMountedCompletedEffects {
        &self.effects
    }

    pub fn cost(&self) -> crate::UiHostPresentationCostReport {
        self.cost
    }

    /// The presented surface, frame, binding, and epoch this completion proves.
    pub fn presented_basis(&self) -> crate::UiHostObservationPresentationBasis {
        crate::UiHostObservationPresentationBasis::new(
            self.work.requirement.host_surface(),
            self.work.frame,
            self.work.requirement.binding(),
            self.epoch,
        )
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::UiHostPresentationEpoch,
        super::UiMountedCompletedEffects,
        crate::UiHostPresentationCostReport,
    ) {
        (self.epoch, self.effects, self.cost)
    }
}

impl std::fmt::Debug for UiMountedSurfacePresentationCompletion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiMountedSurfacePresentationCompletion")
            .field("attempt", &self.work.attempt)
            .field("requirement", &self.work.requirement)
            .field("frame", &self.work.frame)
            .field("content", &self.work.content)
            .field("mode", &self.mode)
            .field("epoch", &self.epoch)
            .field("effects", &self.effects)
            .field("cost", &self.cost)
            .finish_non_exhaustive()
    }
}
