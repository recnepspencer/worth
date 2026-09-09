#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedPresentationAffinity {
    predecessor: Option<crate::UiMountedFrameIdentity>,
    successor: crate::UiMountedFrameIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
    binding: crate::UiSurfaceBindingGeneration,
    content: crate::UiMountedContentGeneration,
    baseline: crate::UiHostSurfaceBaselineIdentity,
    receipt_affinity: Option<crate::UiMountedNodeReceiptAffinity>,
}

pub(super) struct UiMountedPresentationAffinityInput {
    pub predecessor: Option<crate::UiMountedFrameIdentity>,
    pub successor: crate::UiMountedFrameIdentity,
    pub surface: crate::UiSemanticSurfaceIdentity,
    pub binding: crate::UiSurfaceBindingGeneration,
    pub content: crate::UiMountedContentGeneration,
    pub baseline: crate::UiHostSurfaceBaselineIdentity,
    pub receipt_affinity: Option<crate::UiMountedNodeReceiptAffinity>,
}

impl UiMountedPresentationAffinity {
    /// Carries mounted identity into an inert projection without issuing work
    /// or granting publication or settlement authority.
    /// A removal-only frame may have no successor node receipt affinity;
    /// fragments containing successor nodes still require that exact affinity.
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        predecessor: Option<crate::UiMountedFrameIdentity>,
        successor: crate::UiMountedFrameIdentity,
        binding: crate::UiMountedSurfaceBindingRequirement,
        content: crate::UiMountedContentGeneration,
        receipt_affinity: Option<crate::UiMountedNodeReceiptAffinity>,
    ) -> Self {
        Self::from_runtime(UiMountedPresentationAffinityInput {
            predecessor,
            successor,
            surface: binding.semantic_surface(),
            binding: binding.binding(),
            content,
            baseline: binding.baseline(),
            receipt_affinity,
        })
    }

    pub(super) const fn from_runtime(input: UiMountedPresentationAffinityInput) -> Self {
        Self {
            predecessor: input.predecessor,
            successor: input.successor,
            surface: input.surface,
            binding: input.binding,
            content: input.content,
            baseline: input.baseline,
            receipt_affinity: input.receipt_affinity,
        }
    }

    pub const fn predecessor(self) -> Option<crate::UiMountedFrameIdentity> {
        self.predecessor
    }

    pub const fn successor(self) -> crate::UiMountedFrameIdentity {
        self.successor
    }

    pub const fn surface(self) -> crate::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub const fn binding(self) -> crate::UiSurfaceBindingGeneration {
        self.binding
    }

    pub const fn content(self) -> crate::UiMountedContentGeneration {
        self.content
    }

    pub const fn baseline(self) -> crate::UiHostSurfaceBaselineIdentity {
        self.baseline
    }

    pub const fn receipt_affinity(self) -> Option<crate::UiMountedNodeReceiptAffinity> {
        self.receipt_affinity
    }

    pub(super) const fn with_receipt_affinity(
        mut self,
        receipt_affinity: Option<crate::UiMountedNodeReceiptAffinity>,
    ) -> Self {
        self.receipt_affinity = receipt_affinity;
        self
    }
}
