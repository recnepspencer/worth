use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiMountedCoordinateSpace};

use super::logical_rect::{UiLogicalRect, UiTruthGeometryDenial};

/// A rectangle sampled and admitted for presentation against one host basis.
///
/// Only presentation sampling mints it. It is not on screen until an admitted
/// witness displays it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiAcceptedRect {
    rect: UiLogicalRect,
    presentation: UiHostObservationPresentationBasis,
}

impl UiAcceptedRect {
    /// A curve point sampled on `presentation`, in the coordinate space of
    /// the published target it heads for.
    pub(in crate::mounting::presentation) fn sampled(
        components: [f32; 4],
        target: super::UiPublishedRect,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<Self, UiTruthGeometryDenial> {
        UiLogicalRect::new(components, target.coordinate_space())
            .map(|rect| Self { rect, presentation })
    }

    /// A sample drawn in `coordinate_space` with no target to take it from.
    #[cfg(test)]
    pub(in crate::mounting::presentation) fn sampled_for_test(
        components: [f32; 4],
        coordinate_space: UiMountedCoordinateSpace,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<Self, UiTruthGeometryDenial> {
        UiLogicalRect::new(components, coordinate_space).map(|rect| Self { rect, presentation })
    }

    /// A sample on `presentation` that holds its target at published
    /// geometry: a track starting at its predecessor's layout, or resting at
    /// its target's. The sample is what crosses; the rect keeps its place.
    pub(in crate::mounting::presentation) const fn holding(
        published: super::UiPublishedRect,
        presentation: UiHostObservationPresentationBasis,
    ) -> Self {
        Self {
            rect: published.rect(),
            presentation,
        }
    }

    pub(super) const fn rect(self) -> UiLogicalRect {
        self.rect
    }

    /// Another rectangle drawn by the same sample, against its basis.
    pub(super) const fn with_rect(self, rect: UiLogicalRect) -> Self {
        Self { rect, ..self }
    }

    /// The raw components: for the serialization and GPU-upload edges, for
    /// damage regions, and for the two owners that measure a sample against
    /// published geometry: the curve span, and a Scroll group's offset from
    /// its rest content box.
    pub(crate) fn components(self) -> [f32; 4] {
        self.rect.components()
    }

    pub(crate) const fn coordinate_space(self) -> UiMountedCoordinateSpace {
        self.rect.coordinate_space()
    }

    pub(crate) const fn presentation_basis(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    /// Whether this sample puts its target exactly where published truth
    /// placed it: an entrance's first accepted frame stands at its initial
    /// geometry.
    pub(crate) fn stands_at(self, published: super::UiPublishedRect) -> bool {
        self.rect == published.rect()
    }

    /// Whether two accepted samples put their target in the same place,
    /// whatever basis each was sampled against.
    pub(crate) fn occupies_same_rect(self, other: Self) -> bool {
        self.rect == other.rect
    }

    /// The same sample against a later basis of the same surface binding: the
    /// owner's rebase when its presented epoch advances.
    pub(in crate::mounting::presentation) fn rebased(
        self,
        presentation: UiHostObservationPresentationBasis,
    ) -> Result<Self, UiRebaseDenial> {
        if presentation.host_surface() != self.presentation.host_surface() {
            return Err(UiRebaseDenial::SurfaceChanged);
        }
        if presentation.binding() != self.presentation.binding() {
            return Err(UiRebaseDenial::BindingChanged);
        }
        Ok(Self {
            presentation,
            ..self
        })
    }

    /// Rebind to the basis a publication moved this surface to, whatever
    /// binding it names.
    pub(in crate::mounting::presentation) const fn rebound(
        self,
        presentation: UiHostObservationPresentationBasis,
    ) -> Self {
        Self {
            presentation,
            ..self
        }
    }
}

/// Why accepted geometry cannot follow its surface to a new basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiRebaseDenial {
    SurfaceChanged,
    BindingChanged,
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiHostPresentationEpoch, UiHostSurfaceIdentity, UiMountedFrameIdentity,
        UiSurfaceBindingGeneration,
    };

    fn basis(
        surface: UiHostSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        epoch: u64,
    ) -> UiHostObservationPresentationBasis {
        UiHostObservationPresentationBasis::new(
            surface,
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            binding,
            UiHostPresentationEpoch::issued_by_host(epoch),
        )
    }

    #[test]
    fn a_sample_rebases_only_within_its_surface_binding() {
        let (surface, binding) = (
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        );
        let sample = UiAcceptedRect::sampled_for_test(
            [4.0, 8.0, 16.0, 32.0],
            UiMountedCoordinateSpace::Viewport,
            basis(surface, binding, 1),
        )
        .unwrap();
        let later = basis(surface, binding, 2);
        let rebased = sample.rebased(later).expect("a later basis of the binding");
        assert_eq!(rebased.presentation_basis(), later);
        assert!(
            rebased.occupies_same_rect(sample),
            "rebasing never moves it"
        );
        assert_eq!(
            sample.rebased(basis(
                surface,
                UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                2
            )),
            Err(UiRebaseDenial::BindingChanged)
        );
        assert_eq!(
            sample.rebased(basis(
                UiHostSurfaceIdentity::mint_unbound().unwrap(),
                binding,
                2
            )),
            Err(UiRebaseDenial::SurfaceChanged)
        );
    }
}
