//! Where a frame presents laid-out geometry.
//!
//! Layout puts Portal content where its parent lays it out. A frame presents
//! that content elsewhere: moved by the step its Portal moves the content it
//! was fitted to, and nowhere while the Portal presents none of it. Every
//! reader that shows an occurrence's geometry, whether its paint, its hit
//! region, its text or its Scroll region's content and chrome, takes it
//! through the same [`UiMountedPlacement`], and a placement is the only way to
//! move geometry through a Portal: each move takes a [`UiPortalMove`] that
//! only [`UiMountedPlacement::present`] makes.

mod mechanics;
mod scroll_region;
mod spaces;

pub(crate) use scroll_region::UiMountedScrollRegionBoxes;
pub(crate) use spaces::{UiLaidOut, UiPresented};
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedPortalOverlayMechanic};

/// Where one frame presents an occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiMountedPlacement {
    /// Where it is laid out.
    InPlace,
    /// Moved through an open Portal.
    ThroughPortal(UiPortalPresentation),
    /// Portal content the frame presents nowhere.
    Hidden,
}

/// An open Portal and the laid-out box it presents at the origin of its
/// paint bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPortalPresentation {
    portal: UiMountedPortalOverlayMechanic,
    source_anchor: UiMountedCanonicalBox,
}

impl UiPortalPresentation {
    /// `portal` presenting the content it was fitted to, which begins at
    /// `content_anchor`'s origin: not the owner's origin when that content is
    /// laid out away from it.
    pub(in crate::mounting::projection) const fn fitted_to(
        portal: UiMountedPortalOverlayMechanic,
        content_anchor: UiMountedCanonicalBox,
    ) -> Self {
        Self {
            portal,
            source_anchor: content_anchor,
        }
    }
}

/// One move through a Portal, which only [`UiMountedPlacement::present`]
/// makes.
pub(crate) struct UiPortalMove(UiPortalPresentation);

impl UiPortalMove {
    pub(crate) const fn portal(&self) -> UiMountedPortalOverlayMechanic {
        self.0.portal
    }

    /// The laid-out box the Portal presents at the origin of its paint
    /// bounds.
    pub(crate) const fn source_anchor(&self) -> UiMountedCanonicalBox {
        self.0.source_anchor
    }
}

/// Laid-out geometry a Portal can present.
pub(crate) trait UiPortalPresentable: Sized {
    type Denial;

    /// This geometry where `by` presents it: `None` when none of it shows.
    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Self::Denial>;
}

impl UiMountedPlacement {
    /// `laid_out` where this placement presents it: `None` when it presents
    /// none of it.
    pub(crate) fn present<T: UiPortalPresentable>(
        self,
        laid_out: UiLaidOut<T>,
    ) -> Result<Option<UiPresented<T>>, T::Denial> {
        match self {
            Self::InPlace => Ok(Some(UiPresented::placed(laid_out.into_layout_space()))),
            Self::ThroughPortal(presentation) => Ok(laid_out
                .into_layout_space()
                .moved(&UiPortalMove(presentation))?
                .map(UiPresented::placed)),
            Self::Hidden => Ok(None),
        }
    }

    /// The Portal this placement presents through.
    pub(crate) const fn portal(self) -> Option<UiMountedPortalOverlayMechanic> {
        match self {
            Self::ThroughPortal(presentation) => Some(presentation.portal),
            Self::InPlace | Self::Hidden => None,
        }
    }

    /// What of its host surface the Portal this placement presents through
    /// covers: `None` when it presents through no Portal, or through one that
    /// covers none of that surface.
    pub(crate) fn coverage(self) -> Option<UiMountedCanonicalBox> {
        self.portal()
            .and_then(super::appearance::portal_coverage_box)
    }
}
