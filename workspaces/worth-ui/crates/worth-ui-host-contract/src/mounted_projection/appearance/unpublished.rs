use std::collections::HashSet;

use super::{UiMountedAppearanceMechanicIdentity, UiMountedAppearanceWork};
use crate::UiMountedSemanticTextMechanic;

#[path = "unpublished_validation.rs"]
mod validation;

pub const UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiUnpublishedAppearanceFragmentIdentity {
    NodeReceipt {
        predecessor: Option<crate::UiMountedNodeReceiptIdentity>,
        successor: Option<crate::UiMountedNodeReceiptIdentity>,
    },
    SurfaceOverlay(crate::UiSemanticSurfaceIdentity),
    SurfacePointer {
        surface: crate::UiSemanticSurfaceIdentity,
        pointer: crate::UiHostPointerIdentity,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiUnpublishedAppearanceFragment {
    identity: UiUnpublishedAppearanceFragmentIdentity,
    work: UiMountedAppearanceWork,
    text_candidates: Box<[UiMountedSemanticTextMechanic]>,
    surface_binding: crate::UiMountedSurfaceBindingRequirement,
    presentation_affinity: crate::UiMountedPresentationAffinity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiUnpublishedAppearanceFrameProjection {
    frame: crate::UiMountedFrameIdentity,
    presentation: crate::UiMountedPresentationAttemptIdentity,
    fragments: Box<[UiUnpublishedAppearanceFragment]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiUnpublishedAppearanceFrameProjectionDenial {
    EmptyFragments,
    FragmentCapacityExceeded,
    TextCandidateCapacityExceeded,
    FrameMismatch,
    PresentationMismatch,
    SurfaceBindingMismatch,
    PresentationAffinityMismatch,
    FragmentIdentityMismatch,
    NodeReceiptFrameMismatch,
    NodeReceiptAttributionMissing,
    NodeReceiptInstanceMismatch,
    NodeReceiptPredecessorFrameMismatch,
    NodeReceiptPredecessorMismatch,
    NodeReceiptSuccessorFrameMismatch,
    NodeReceiptSuccessorMismatch,
    TextCandidateFrameMismatch,
    TextCandidateSurfaceMismatch,
    TextCandidateBindingMismatch,
    TextCandidateContentMismatch,
    TextCandidateReceiptMismatch,
    TextCandidateSpanMismatch,
    TextCandidateRequiresNodeReceipt,
    OverlayParticipantMismatch,
    DuplicateFragmentIdentity(UiUnpublishedAppearanceFragmentIdentity),
    ConflictingMechanicIdentity(UiMountedAppearanceMechanicIdentity),
    MultipleOverlayFragmentsForSurface(crate::UiSemanticSurfaceIdentity),
    MultiplePointerFragmentsForSurface(crate::UiSemanticSurfaceIdentity),
    MultiplePointerMechanicsForSurface(crate::UiSemanticSurfaceIdentity),
    DuplicateNodeAttribution(crate::UiMountedInstanceIdentity),
    CandidateCapabilityGenerationMismatch,
    CandidateCapabilityProfileDigestMismatch,
    PresentationReceiptAffinityMismatch,
    ConflictingTextCandidate {
        receipt: crate::UiMountedNodeReceiptIdentity,
        span: crate::UiMountedTextPaintSpanIdentity,
    },
}

impl UiUnpublishedAppearanceFragment {
    /// Builds an inert fragment from runtime-issued work and affinities.
    ///
    /// This carries no publication, settlement, or compare-and-swap authority.
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        identity: UiUnpublishedAppearanceFragmentIdentity,
        work: UiMountedAppearanceWork,
        text_candidates: impl IntoIterator<Item = UiMountedSemanticTextMechanic>,
        surface_binding: crate::UiMountedSurfaceBindingRequirement,
        presentation_affinity: crate::UiMountedPresentationAffinity,
    ) -> Result<Self, UiUnpublishedAppearanceFrameProjectionDenial> {
        let mut admitted_candidates =
            Vec::with_capacity(crate::UiMountedSemanticTextTable::MAX_ROWS + 1);
        for candidate in text_candidates
            .into_iter()
            .take(crate::UiMountedSemanticTextTable::MAX_ROWS + 1)
        {
            admitted_candidates.push(candidate);
        }
        let text_candidates = admitted_candidates;
        if text_candidates.len() > crate::UiMountedSemanticTextTable::MAX_ROWS {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::TextCandidateCapacityExceeded,
            );
        }
        let fragment = Self {
            identity,
            work,
            text_candidates: text_candidates.into_boxed_slice(),
            surface_binding,
            presentation_affinity,
        };
        validation::validate_fragment(
            &fragment,
            fragment.work.successor().frame(),
            fragment.work.successor().overlay_order().presentation(),
        )?;
        Ok(fragment)
    }

    pub const fn identity(&self) -> UiUnpublishedAppearanceFragmentIdentity {
        self.identity
    }

    pub const fn work(&self) -> &UiMountedAppearanceWork {
        &self.work
    }

    pub fn text_candidates(&self) -> &[UiMountedSemanticTextMechanic] {
        &self.text_candidates
    }

    pub const fn surface_binding(&self) -> crate::UiMountedSurfaceBindingRequirement {
        self.surface_binding
    }

    pub const fn presentation_affinity(&self) -> crate::UiMountedPresentationAffinity {
        self.presentation_affinity
    }
}

impl UiUnpublishedAppearanceFrameProjection {
    /// Builds a read-only host projection; it does not publish or settle work.
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        frame: crate::UiMountedFrameIdentity,
        presentation: crate::UiMountedPresentationAttemptIdentity,
        fragments: impl IntoIterator<Item = UiUnpublishedAppearanceFragment>,
    ) -> Result<Self, UiUnpublishedAppearanceFrameProjectionDenial> {
        let mut admitted_fragments =
            Vec::with_capacity(UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY + 1);
        for fragment in fragments
            .into_iter()
            .take(UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY + 1)
        {
            admitted_fragments.push(fragment);
        }
        let fragments = admitted_fragments;
        if fragments.is_empty() {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::EmptyFragments);
        }
        if fragments.len() > UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentCapacityExceeded);
        }

        let mut fragment_identities = HashSet::new();
        let mut overlay_surfaces = HashSet::new();
        let mut pointer_surfaces = HashSet::new();
        let mut node_attributions = HashSet::new();
        let mut mechanic_identities = HashSet::new();
        for fragment in &fragments {
            validation::validate_fragment(fragment, frame, presentation)?;
            if !fragment_identities.insert(fragment.identity) {
                return Err(
                    UiUnpublishedAppearanceFrameProjectionDenial::DuplicateFragmentIdentity(
                        fragment.identity,
                    ),
                );
            }
            if let UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface) =
                fragment.identity
            {
                if !overlay_surfaces.insert(surface) {
                    return Err(UiUnpublishedAppearanceFrameProjectionDenial::MultipleOverlayFragmentsForSurface(surface));
                }
            }
            if let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor,
                successor,
            } = fragment.identity
            {
                let instance = successor
                    .or(predecessor)
                    .expect("validated node attribution has a receipt")
                    .mounted_instance();
                if !node_attributions.insert(instance) {
                    return Err(
                        UiUnpublishedAppearanceFrameProjectionDenial::DuplicateNodeAttribution(
                            instance,
                        ),
                    );
                }
            }
            if let UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { surface, .. } =
                fragment.identity
            {
                if !pointer_surfaces.insert(surface) {
                    return Err(UiUnpublishedAppearanceFrameProjectionDenial::MultiplePointerFragmentsForSurface(surface));
                }
            }
            for identity in validation::fragment_mechanic_identities(fragment) {
                if !mechanic_identities.insert(identity.clone()) {
                    return Err(
                        UiUnpublishedAppearanceFrameProjectionDenial::ConflictingMechanicIdentity(
                            identity,
                        ),
                    );
                }
            }
        }
        Ok(Self {
            frame,
            presentation,
            fragments: fragments.into_boxed_slice(),
        })
    }

    pub const fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.frame
    }

    pub const fn presentation(&self) -> crate::UiMountedPresentationAttemptIdentity {
        self.presentation
    }

    pub fn fragments(&self) -> &[UiUnpublishedAppearanceFragment] {
        &self.fragments
    }
}

#[cfg(test)]
#[path = "unpublished_tests.rs"]
mod tests;
