use super::displayed::UiDisplayedRect;
use super::published::UiPublishedRect;
use crate::mounting::presentation::UiDisplayedSurfaceBasis;
use crate::runtime::scroll::UiScrollOffset;

/// Where an admitted witness proved a Scroll content group on screen: the
/// distance from the group's published rest box to its displayed sample.
///
/// Only [`UiDisplayedScrollOffset::from_rest`] mints it, from a displayed
/// rectangle, so it always carries the displayed basis that proved it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiDisplayedScrollOffset {
    inline_subpixels: i64,
    block_subpixels: i64,
    displayed: UiDisplayedSurfaceBasis,
}

/// Why a displayed sample does not stand at any scroll offset from its rest
/// box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollStandingDenial {
    /// The sample and the rest box are measured in different spaces.
    CoordinateSpaceChanged,
    /// The sample sits before rest, which no non-negative offset describes.
    BeforeRest,
}

impl UiDisplayedScrollOffset {
    /// The offset `sample` stands at. The rest box is the group's committed
    /// content box: an applied pose moves the owner's descendants, never the
    /// owner, so rest is where the content sits at the origin offset.
    pub(crate) fn from_rest(
        rest: UiPublishedRect,
        sample: UiDisplayedRect,
    ) -> Result<Self, UiScrollStandingDenial> {
        let (rest, shown) = (rest.rect(), sample.rect());
        if rest.coordinate_space() != shown.coordinate_space() {
            return Err(UiScrollStandingDenial::CoordinateSpaceChanged);
        }
        let ([rest_x, rest_y, ..], [shown_x, shown_y, ..]) =
            (rest.components(), shown.components());
        let inline_subpixels = subpixels(rest_x - shown_x);
        let block_subpixels = subpixels(rest_y - shown_y);
        if inline_subpixels < 0 || block_subpixels < 0 {
            return Err(UiScrollStandingDenial::BeforeRest);
        }
        Ok(Self {
            inline_subpixels,
            block_subpixels,
            displayed: sample.displayed_basis(),
        })
    }

    /// The settle crossing: the published offset mounted geometry and the
    /// Scroll owner take on once they follow what the witness displayed.
    pub(crate) fn settled(self) -> UiScrollOffset {
        UiScrollOffset::new(self.inline_subpixels, self.block_subpixels)
            .expect("a displayed scroll offset is non-negative when minted")
    }

    /// Whether published truth already stands where this was displayed.
    pub(crate) fn stands_at(self, published: UiScrollOffset) -> bool {
        self.settled() == published
    }

    pub(crate) const fn displayed_basis(self) -> UiDisplayedSurfaceBasis {
        self.displayed
    }
}

/// A displayed offset minted the way production mints one: an accepted sample
/// `offset` above a rest box at the origin, displayed by an admitted witness.
#[cfg(test)]
pub(crate) fn displayed_scroll_offset_for_test(offset: UiScrollOffset) -> UiDisplayedScrollOffset {
    use worth_ui_host_contract::UiMountedCoordinateSpace::Viewport;
    let points = |subpixels: i64| {
        subpixels as f32
            / worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32
    };
    let basis = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );
    let sample = super::UiAcceptedRect::sampled_for_test(
        [
            -points(offset.inline_subpixels()),
            -points(offset.block_subpixels()),
            1.0,
            1.0,
        ],
        Viewport,
        basis,
    )
    .unwrap();
    let witness = crate::mounting::presentation::presented_surface_witness_for_certification(basis);
    let displayed = UiDisplayedRect::displayed(sample, witness.displayed_basis()).unwrap();
    let rest = UiPublishedRect::from_committed_components([0.0, 0.0, 1.0, 1.0], Viewport).unwrap();
    let minted = UiDisplayedScrollOffset::from_rest(rest, displayed).unwrap();
    assert_eq!(minted.settled(), offset);
    minted
}

fn subpixels(logical_points: f32) -> i64 {
    (logical_points * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32)
        .round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mounting::presentation::{
        presented_surface_witness_for_certification, UiAcceptedRect,
    };
    use worth_ui_host_contract::{
        UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiMountedCoordinateSpace, UiMountedFrameIdentity, UiSurfaceBindingGeneration,
        UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as PER_POINT,
    };

    const SPACE: UiMountedCoordinateSpace = UiMountedCoordinateSpace::Viewport;

    fn displayed_at(x: f32, y: f32, space: UiMountedCoordinateSpace) -> UiDisplayedRect {
        let basis = UiHostObservationPresentationBasis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            UiHostPresentationEpoch::issued_by_host(1),
        );
        let accepted = UiAcceptedRect::sampled_for_test([x, y, 40.0, 80.0], space, basis).unwrap();
        let witness = presented_surface_witness_for_certification(basis);
        UiDisplayedRect::displayed(accepted, witness.displayed_basis()).unwrap()
    }

    #[test]
    fn a_displayed_sample_stands_at_its_distance_from_rest() {
        let rest =
            UiPublishedRect::from_committed_components([10.0, 30.0, 40.0, 80.0], SPACE).unwrap();
        let scrolled = UiDisplayedScrollOffset::from_rest(rest, displayed_at(10.0, 18.0, SPACE))
            .expect("content sampled above rest is scrolled content");
        assert_eq!(
            scrolled.settled(),
            UiScrollOffset::new(0, 12 * PER_POINT).unwrap()
        );
        assert!(scrolled.stands_at(UiScrollOffset::new(0, 12 * PER_POINT).unwrap()));
        assert!(!scrolled.stands_at(UiScrollOffset::origin()));
        let at_rest =
            UiDisplayedScrollOffset::from_rest(rest, displayed_at(10.0, 30.0, SPACE)).unwrap();
        assert_eq!(at_rest.settled(), UiScrollOffset::origin());
        assert_eq!(
            UiDisplayedScrollOffset::from_rest(rest, displayed_at(10.0, 42.0, SPACE)),
            Err(UiScrollStandingDenial::BeforeRest)
        );
        assert_eq!(
            UiDisplayedScrollOffset::from_rest(
                rest,
                displayed_at(10.0, 18.0, UiMountedCoordinateSpace::HostSurface)
            ),
            Err(UiScrollStandingDenial::CoordinateSpaceChanged)
        );
    }
}
