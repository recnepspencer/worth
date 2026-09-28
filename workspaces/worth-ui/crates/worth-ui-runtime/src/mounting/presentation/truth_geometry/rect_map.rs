use super::accepted::UiAcceptedRect;
use super::displayed::UiDisplayedRect;
use super::published::UiPublishedRect;

/// How a Motion sample placed a target whose published geometry is `source`.
/// Every published rect laid out in that geometry moves with it, so the map
/// carries published geometry to the accepted place the sample put it. Only
/// an accepted sample, which presentation sampling alone mints, supplies the
/// destination.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPublishedToAcceptedMap {
    source: UiPublishedRect,
    sampled: UiAcceptedRect,
}

/// How a prepared entrance places a target whose committed geometry is
/// `source` at its committed `initial` geometry. Both ends are published.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPublishedMap {
    source: UiPublishedRect,
    initial: UiPublishedRect,
}

impl UiPublishedToAcceptedMap {
    /// The map `sampled` makes of `source`. A source without area has no
    /// extent to scale what is laid out in it by, so it carries nothing.
    pub(crate) fn of_sample(source: UiPublishedRect, sampled: UiAcceptedRect) -> Option<Self> {
        source.has_area().then_some(Self { source, sampled })
    }

    /// Where the sample put `rect`. The rect must share the source's space.
    /// `None` when the sample carries it beyond finite geometry.
    pub(crate) fn apply(self, rect: UiPublishedRect) -> Option<UiAcceptedRect> {
        let mapped = rect
            .rect()
            .mapped(self.source.rect(), self.sampled.rect())?;
        Some(self.sampled.with_rect(mapped))
    }
}

impl UiPublishedMap {
    /// The map from `source` to `initial`. A source without area has no
    /// extent to scale what is laid out in it by, so it carries nothing.
    pub(crate) fn between(source: UiPublishedRect, initial: UiPublishedRect) -> Option<Self> {
        source.has_area().then_some(Self { source, initial })
    }

    /// Where the entrance puts `rect`. The rect must share the source's space.
    /// `None` when the entrance carries it beyond finite geometry.
    pub(crate) fn apply(self, rect: UiPublishedRect) -> Option<UiPublishedRect> {
        let mapped = rect
            .rect()
            .mapped(self.source.rect(), self.initial.rect())?;
        Some(UiPublishedRect::from_rect(mapped))
    }
}

impl UiPublishedRect {
    /// The named displayed-to-published crossing: the rect an owner commits
    /// from what an admitted witness displayed, as a Portal anchors its
    /// placement or Scroll anchors its offset to where the host showed the
    /// target. The displayed rect is the evidence; nothing else crosses.
    pub(crate) fn adopting(displayed: UiDisplayedRect) -> Self {
        Self::from_rect(displayed.rect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::UiMountedCoordinateSpace;

    fn rect(components: [f32; 4]) -> UiPublishedRect {
        UiPublishedRect::from_committed_components(components, UiMountedCoordinateSpace::Viewport)
            .unwrap()
    }

    #[test]
    fn a_source_without_area_carries_nothing() {
        let initial = rect([0.0, 0.0, 20.0, 20.0]);
        assert_eq!(
            UiPublishedMap::between(rect([0.0, 0.0, 0.0, 10.0]), initial),
            None
        );
        assert_eq!(
            UiPublishedMap::between(rect([0.0, 0.0, 10.0, 0.0]), initial),
            None
        );
        let map = UiPublishedMap::between(rect([0.0, 0.0, 10.0, 10.0]), initial)
            .expect("a source with area carries its layout");
        assert_eq!(
            map.apply(rect([5.0, 5.0, 5.0, 5.0])),
            Some(rect([10.0, 10.0, 10.0, 10.0]))
        );
    }

    #[test]
    fn a_map_carrying_a_rect_beyond_finite_geometry_places_it_nowhere() {
        let sliver = rect([0.0, 0.0, 1.0e-30, 10.0]);
        let map = UiPublishedMap::between(sliver, rect([0.0, 0.0, 1.0e10, 10.0]))
            .expect("a sliver has area");
        assert_eq!(map.apply(rect([0.0, 0.0, 1.0e30, 10.0])), None);
    }
}
