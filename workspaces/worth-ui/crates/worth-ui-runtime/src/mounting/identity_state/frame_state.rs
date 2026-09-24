//! The mounted frame this identity world currently answers for.
//!
//! A frame is either published, owning its projection and every publication
//! part together, or unpublished: identity progression revoked the published
//! projection while the next one still needs the semantic, appearance and
//! pointer predecessors it left behind. A frame candidate may already own node
//! receipts for the unpublished frame; whether it does is independent of
//! whether any projection ever preceded it.

use std::rc::Rc;

use worth_ui_host_contract::{
    UiMountedFrameCanonicalCore, UiMountedFrameIdentity, UiMountedFrameManifest,
    UiSemanticSurfaceIdentity,
};

use crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource;
use crate::mounting::projection::{
    UiMountedAppearanceFrameState, UiMountedPointerAffordanceState, UiMountedSemanticProjection,
};
use crate::mounting::{
    UiMountedFramePublicationReceipt, UiMountedFrameReuseContract, UiMountedNodeReceiptBasis,
    UiMountedProjectionFrameOwner,
};

pub(super) enum UiMountedFrameState {
    Unpublished {
        receipts: Option<UiMountedNodeReceiptBasis>,
        predecessors: Option<UiUnprojectedPredecessors>,
    },
    Published(UiPublishedMountedFrame),
}

/// What the last published projection leaves for the next one to succeed.
/// A graph replacement keeps the physical predecessors but leaves no semantic
/// one: the replaced graph's semantic projection describes nothing in the new.
#[derive(Clone)]
pub(super) struct UiUnprojectedPredecessors {
    semantic: Option<UiMountedSemanticProjection>,
    appearance: UiMountedAppearanceFrameState,
    pointer: UiMountedPointerAffordanceState,
}

pub(super) struct UiPublishedMountedFrame {
    pub(super) receipts: UiMountedNodeReceiptBasis,
    pub(super) projection: Rc<UiMountedProjectionFrameOwner>,
    pub(super) manifest: UiMountedFrameManifest,
    pub(super) core: UiMountedFrameCanonicalCore,
    pub(super) publication: UiMountedFramePublicationReceipt,
    pub(super) trace_source: WorthUiPreparedVisualTraceSource,
    pub(super) reuse_contract: UiMountedFrameReuseContract,
}

impl UiMountedFrameState {
    pub(super) const fn vacant() -> Self {
        Self::Unpublished {
            receipts: None,
            predecessors: None,
        }
    }

    pub(super) fn published(&self) -> Option<&UiPublishedMountedFrame> {
        match self {
            Self::Published(published) => Some(published),
            Self::Unpublished { .. } => None,
        }
    }

    pub(super) fn receipts(&self) -> Option<&UiMountedNodeReceiptBasis> {
        match self {
            Self::Published(published) => Some(&published.receipts),
            Self::Unpublished { receipts, .. } => receipts.as_ref(),
        }
    }

    pub(super) fn receipts_mut(&mut self) -> Option<&mut UiMountedNodeReceiptBasis> {
        match self {
            Self::Published(published) => Some(&mut published.receipts),
            Self::Unpublished { receipts, .. } => receipts.as_mut(),
        }
    }

    pub(super) fn frame(&self) -> Option<UiMountedFrameIdentity> {
        self.receipts().map(UiMountedNodeReceiptBasis::frame)
    }

    pub(super) fn projection(&self) -> Option<&UiMountedProjectionFrameOwner> {
        self.published().map(|published| &*published.projection)
    }

    pub(super) fn projection_mut(&mut self) -> Option<&mut Rc<UiMountedProjectionFrameOwner>> {
        match self {
            Self::Published(published) => Some(&mut published.projection),
            Self::Unpublished { .. } => None,
        }
    }

    pub(super) fn semantic_predecessor(&self) -> Option<&UiMountedSemanticProjection> {
        match self {
            Self::Published(published) => {
                Some(published.projection.projection().semantic_projection())
            }
            Self::Unpublished { predecessors, .. } => {
                predecessors.as_ref().and_then(|p| p.semantic.as_ref())
            }
        }
    }

    pub(super) fn appearance_predecessor(&self) -> Option<&UiMountedAppearanceFrameState> {
        match self {
            Self::Published(published) => Some(published.projection.appearance()),
            Self::Unpublished { predecessors, .. } => predecessors.as_ref().map(|p| &p.appearance),
        }
    }

    pub(super) fn pointer_predecessor(&self) -> Option<&UiMountedPointerAffordanceState> {
        match self {
            Self::Published(published) => Some(&published.projection.pointer),
            Self::Unpublished { predecessors, .. } => predecessors.as_ref().map(|p| &p.pointer),
        }
    }

    /// What succeeding this frame state inherits, each physical predecessor
    /// carried through its own successor rule.
    fn predecessors_with(
        &self,
        semantic: Option<UiMountedSemanticProjection>,
        appearance: impl FnOnce(&UiMountedAppearanceFrameState) -> UiMountedAppearanceFrameState,
        pointer: impl FnOnce(&UiMountedPointerAffordanceState) -> UiMountedPointerAffordanceState,
    ) -> Option<UiUnprojectedPredecessors> {
        Some(UiUnprojectedPredecessors {
            semantic,
            appearance: appearance(self.appearance_predecessor()?),
            pointer: pointer(self.pointer_predecessor()?),
        })
    }

    /// A candidate frame owns `receipts` before any projection presents it.
    pub(super) fn advance_unpublished(&mut self, receipts: UiMountedNodeReceiptBasis) {
        let predecessors = match std::mem::replace(self, Self::vacant()) {
            Self::Published(published) => Some(UiUnprojectedPredecessors {
                semantic: Some(
                    published
                        .projection
                        .projection()
                        .semantic_projection()
                        .clone(),
                ),
                appearance: published.projection.appearance().clone(),
                pointer: published.projection.pointer.clone(),
            }),
            Self::Unpublished { predecessors, .. } => predecessors,
        };
        *self = Self::Unpublished {
            receipts: Some(receipts),
            predecessors,
        };
    }

    /// Deregistering `surface` revokes the frame; its predecessors forget it.
    pub(super) fn revoke_for_surface_deregistration(&mut self, surface: UiSemanticSurfaceIdentity) {
        let predecessors = self.predecessors_with(
            self.semantic_predecessor().cloned(),
            |appearance| appearance.after_surface_deregistration(surface),
            |pointer| pointer.after_surface_deregistration(surface),
        );
        *self = Self::Unpublished {
            receipts: None,
            predecessors,
        };
    }

    /// The successor world's frame state: nothing published, and only the
    /// physical predecessors survive.
    pub(super) fn after_graph_replacement(&self) -> Self {
        Self::Unpublished {
            receipts: None,
            predecessors: self.predecessors_with(None, Clone::clone, Clone::clone),
        }
    }

    pub(super) fn publish(&mut self, published: UiPublishedMountedFrame) {
        debug_assert_eq!(published.receipts.frame(), published.publication.frame());
        *self = Self::Published(published);
    }
}
