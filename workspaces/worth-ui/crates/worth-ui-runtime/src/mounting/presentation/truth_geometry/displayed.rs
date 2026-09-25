use worth_ui_host_contract::UiMountedCoordinateSpace;

use super::accepted::UiAcceptedRect;
use super::logical_rect::UiLogicalRect;
use crate::mounting::presentation::UiDisplayedSurfaceBasis;

/// A rectangle an admitted host acknowledgement proved on screen.
///
/// The only way in is [`UiDisplayedRect::displayed`], which consumes the
/// displayed basis of an admitted witness for the surface binding the accepted
/// sample was drawn against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiDisplayedRect {
    rect: UiLogicalRect,
    displayed: UiDisplayedSurfaceBasis,
}

/// Why an accepted rectangle is not what a witness displayed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiDisplayCrossingDenial {
    SurfaceChanged,
    BindingChanged,
}

impl UiDisplayedRect {
    /// The accepted sample the witness's surface binding put on screen.
    ///
    /// The frame may be later than the sample's: a settled sample stays on
    /// screen for every frame of its binding until a later sample replaces it.
    pub(crate) fn displayed(
        accepted: UiAcceptedRect,
        displayed: UiDisplayedSurfaceBasis,
    ) -> Result<Self, UiDisplayCrossingDenial> {
        let sampled = accepted.presentation_basis();
        if sampled.host_surface() != displayed.host_surface() {
            return Err(UiDisplayCrossingDenial::SurfaceChanged);
        }
        if sampled.binding() != displayed.binding() {
            return Err(UiDisplayCrossingDenial::BindingChanged);
        }
        Ok(Self {
            rect: accepted.rect(),
            displayed,
        })
    }

    pub(super) const fn rect(self) -> UiLogicalRect {
        self.rect
    }

    pub(crate) fn components(self) -> [f32; 4] {
        self.rect.components()
    }

    /// Whether the point a platform event reports lands where the host
    /// showed this rect: the platform-event edge.
    pub(crate) fn admits_platform_point(self, point: [f32; 2]) -> bool {
        self.rect.admits(point)
    }

    pub(crate) const fn coordinate_space(self) -> UiMountedCoordinateSpace {
        self.rect.coordinate_space()
    }

    pub(crate) const fn displayed_basis(self) -> UiDisplayedSurfaceBasis {
        self.displayed
    }
}

/// A displayed rect minted the way production mints one: an accepted sample
/// drawn in `coordinate_space` against a fresh basis, displayed by an admitted
/// witness for it.
#[cfg(test)]
pub(crate) fn displayed_rect_for_test(
    components: [f32; 4],
    coordinate_space: UiMountedCoordinateSpace,
) -> UiDisplayedRect {
    let basis = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );
    let sample = UiAcceptedRect::sampled_for_test(components, coordinate_space, basis).unwrap();
    let witness = crate::mounting::presentation::presented_surface_witness_for_certification(basis);
    UiDisplayedRect::displayed(sample, witness.displayed_basis()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiMountedFrameIdentity, UiSurfaceBindingGeneration,
    };

    fn basis(
        surface: UiHostSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
    ) -> UiHostObservationPresentationBasis {
        UiHostObservationPresentationBasis::new(
            surface,
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            binding,
            UiHostPresentationEpoch::issued_by_host(1),
        )
    }

    fn witnessed(basis: UiHostObservationPresentationBasis) -> UiDisplayedSurfaceBasis {
        crate::mounting::presentation::presented_surface_witness_for_certification(basis)
            .displayed_basis()
    }

    #[test]
    fn a_witness_displays_only_samples_of_its_own_surface_binding() {
        let (surface, binding) = (
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        );
        let sample = UiAcceptedRect::sampled_for_test(
            [0.0, 0.0, 10.0, 10.0],
            UiMountedCoordinateSpace::Viewport,
            basis(surface, binding),
        )
        .unwrap();
        assert!(
            UiDisplayedRect::displayed(sample, witnessed(basis(surface, binding))).is_ok(),
            "a later frame of the same binding still shows a settled sample"
        );
        assert_eq!(
            UiDisplayedRect::displayed(
                sample,
                witnessed(basis(
                    UiHostSurfaceIdentity::mint_unbound().unwrap(),
                    binding
                )),
            ),
            Err(UiDisplayCrossingDenial::SurfaceChanged)
        );
        assert_eq!(
            UiDisplayedRect::displayed(
                sample,
                witnessed(basis(
                    surface,
                    UiSurfaceBindingGeneration::mint_unbound().unwrap()
                )),
            ),
            Err(UiDisplayCrossingDenial::BindingChanged)
        );
    }
}
