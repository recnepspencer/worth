use crate::mounting::presentation::UiPublishedRect;
use crate::mounting::UiPresentedHitRect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentedInteractionGeometry {
    bounds: UiPresentedHitRect,
    clip_bounds: UiPresentedHitRect,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentedViewportGeometry {
    bounds: UiPublishedRect,
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
}

// Truth geometry rejects non-finite components, so equality is reflexive.
impl Eq for UiPresentedInteractionGeometry {}
impl Eq for UiPresentedViewportGeometry {}

impl UiPresentedInteractionGeometry {
    pub(super) const fn from_presented_hit_test(
        row: crate::mounting::UiPresentedHitTestRow,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        Self {
            bounds: row.bounds(),
            clip_bounds: row.clip_bounds(),
            presentation,
        }
    }

    pub(crate) const fn bounds(self) -> UiPresentedHitRect {
        self.bounds
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn clip_bounds(self) -> UiPresentedHitRect {
        self.clip_bounds
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn for_test(
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        Self::for_test_with_components(
            presentation,
            [8.0, 8.0, 40.0, 24.0],
            [0.0, 0.0, 960.0, 600.0],
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn for_test_with_components(
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        bounds: [f32; 4],
        clip_bounds: [f32; 4],
    ) -> Self {
        let canonicalize = |components: [f32; 4]| {
            worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
                worth_ui_host_contract::UiMountedCanonicalBoxInput {
                    x: components[0],
                    y: components[1],
                    width: components[2],
                    height: components[3],
                    coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
                },
            )
            .map(UiPublishedRect::from_committed_box)
            .expect("test geometry is canonical")
        };
        Self {
            bounds: UiPresentedHitRect::Published(canonicalize(bounds)),
            clip_bounds: UiPresentedHitRect::Published(canonicalize(clip_bounds)),
            presentation,
        }
    }
}

impl UiPresentedViewportGeometry {
    pub(crate) fn from_current_interaction(
        bounds: UiPublishedRect,
        interaction: UiPresentedInteractionGeometry,
    ) -> Option<Self> {
        (bounds.coordinate_space() == worth_ui_host_contract::UiMountedCoordinateSpace::Viewport)
            .then_some(Self {
                bounds,
                presentation: interaction.presentation(),
            })
    }

    pub(crate) const fn bounds(self) -> UiPublishedRect {
        self.bounds
    }

    pub(crate) const fn presentation(
        self,
    ) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
        self.presentation
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn for_test(
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Self {
        Self {
            bounds: UiPublishedRect::from_committed_box(bounds),
            presentation,
        }
    }
}
