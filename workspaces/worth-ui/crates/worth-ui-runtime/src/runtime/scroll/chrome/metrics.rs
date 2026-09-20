//! The declared scroll-chrome metrics and their admission.
//!
//! Milestone 3.16.1 declares a reserved 12-point gutter, a centered 6-point
//! rounded thumb and a 24-point minimum thumb length. These are declared
//! product metrics, not host defaults, so they enter the runtime through a
//! typed admission that rejects negative extents and any combination that
//! leaves no usable thumb range.

/// Reserved gutter along the scrolled edge, in logical points.
pub(crate) const UI_SCROLL_GUTTER_LOGICAL_POINTS: f32 = 12.0;

/// Thumb thickness across the gutter, in logical points. Centered in the gutter.
pub(crate) const UI_SCROLL_THUMB_LOGICAL_POINTS: f32 = 6.0;

/// Shortest thumb the chrome may present along its track, in logical points.
pub(crate) const UI_SCROLL_THUMB_MINIMUM_LOGICAL_POINTS: f32 = 24.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeMetricsDenial {
    NonFiniteExtent,
    NegativeExtent,
    /// A zero gutter, a zero thumb thickness or a zero minimum length leaves no
    /// thumb a viewer can see or a pointer can grab.
    ThumbRangeUnusable,
    /// A thumb thicker than its gutter cannot be centered inside the reserved
    /// strip, so the gutter would stop being a stable reservation.
    ThumbThicknessExceedsGutter,
}

/// Admitted chrome metrics. Construction is the only place extents are
/// validated; every geometry step downstream consumes this proof.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollChromeMetrics {
    gutter_logical_points: f32,
    thumb_thickness_logical_points: f32,
    minimum_thumb_length_logical_points: f32,
}

impl UiScrollChromeMetrics {
    /// The metrics milestone 3.16.1 declares for this platform.
    pub(crate) fn declared() -> Self {
        Self::admit(
            UI_SCROLL_GUTTER_LOGICAL_POINTS,
            UI_SCROLL_THUMB_LOGICAL_POINTS,
            UI_SCROLL_THUMB_MINIMUM_LOGICAL_POINTS,
        )
        .expect("declared 3.16.1 chrome metrics are admissible")
    }

    pub(crate) fn admit(
        gutter_logical_points: f32,
        thumb_thickness_logical_points: f32,
        minimum_thumb_length_logical_points: f32,
    ) -> Result<Self, UiScrollChromeMetricsDenial> {
        let extents = [
            gutter_logical_points,
            thumb_thickness_logical_points,
            minimum_thumb_length_logical_points,
        ];
        if extents.iter().any(|extent| !extent.is_finite()) {
            return Err(UiScrollChromeMetricsDenial::NonFiniteExtent);
        }
        if extents.iter().any(|extent| *extent < 0.0) {
            return Err(UiScrollChromeMetricsDenial::NegativeExtent);
        }
        if extents.contains(&0.0) {
            return Err(UiScrollChromeMetricsDenial::ThumbRangeUnusable);
        }
        if thumb_thickness_logical_points > gutter_logical_points {
            return Err(UiScrollChromeMetricsDenial::ThumbThicknessExceedsGutter);
        }
        Ok(Self {
            gutter_logical_points,
            thumb_thickness_logical_points,
            minimum_thumb_length_logical_points,
        })
    }

    pub(crate) const fn gutter_logical_points(self) -> f32 {
        self.gutter_logical_points
    }

    pub(crate) const fn thumb_thickness_logical_points(self) -> f32 {
        self.thumb_thickness_logical_points
    }

    pub(crate) const fn minimum_thumb_length_logical_points(self) -> f32 {
        self.minimum_thumb_length_logical_points
    }

    /// Distance from the gutter edge to the thumb edge that centers the thumb.
    pub(crate) fn thumb_centering_inset_logical_points(self) -> f32 {
        (self.gutter_logical_points - self.thumb_thickness_logical_points) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_metrics_are_the_milestone_numbers_and_center_the_thumb() {
        let metrics = UiScrollChromeMetrics::declared();
        assert_eq!(metrics.gutter_logical_points(), 12.0);
        assert_eq!(metrics.thumb_thickness_logical_points(), 6.0);
        assert_eq!(metrics.minimum_thumb_length_logical_points(), 24.0);
        assert_eq!(metrics.thumb_centering_inset_logical_points(), 3.0);
    }

    #[test]
    fn admission_refuses_negative_zero_and_overthick_thumb_metrics() {
        assert_eq!(
            UiScrollChromeMetrics::admit(-1.0, 6.0, 24.0),
            Err(UiScrollChromeMetricsDenial::NegativeExtent)
        );
        assert_eq!(
            UiScrollChromeMetrics::admit(12.0, 0.0, 24.0),
            Err(UiScrollChromeMetricsDenial::ThumbRangeUnusable)
        );
        assert_eq!(
            UiScrollChromeMetrics::admit(12.0, 6.0, 0.0),
            Err(UiScrollChromeMetricsDenial::ThumbRangeUnusable)
        );
        assert_eq!(
            UiScrollChromeMetrics::admit(6.0, 12.0, 24.0),
            Err(UiScrollChromeMetricsDenial::ThumbThicknessExceedsGutter)
        );
        assert_eq!(
            UiScrollChromeMetrics::admit(f32::NAN, 6.0, 24.0),
            Err(UiScrollChromeMetricsDenial::NonFiniteExtent)
        );
    }
}
