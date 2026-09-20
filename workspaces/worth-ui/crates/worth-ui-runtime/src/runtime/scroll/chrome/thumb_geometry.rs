//! Thumb length and position along one track.
//!
//! Length reflects the viewport/content proportion of the track, clamped up to
//! the admitted minimum length. Clamping the length shortens the distance the
//! thumb can travel, so travel is *compressed* rather than truncated: the thumb
//! still reaches both ends of its track exactly at offset zero and at the
//! bound. Position comes from the accepted displayed offset the caller passes
//! in — never from a pending transition target.

/// The thumb's resolved extent along its track. `travel` is the compressed
/// distance between the thumb's start at offset zero and its start at the
/// bound, so `length + travel == track length`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollThumbExtent {
    length_logical_points: f32,
    travel_logical_points: f32,
}

/// `None` when the axis has no overflow, because a no-overflow axis has no
/// thumb and therefore no drag target.
pub(crate) fn thumb_extent(
    track_length_logical_points: f32,
    viewport_extent_logical_points: f32,
    max_offset_subpixels: i64,
    metrics: super::UiScrollChromeMetrics,
) -> Option<UiScrollThumbExtent> {
    if max_offset_subpixels <= 0 {
        return None;
    }
    if !track_length_logical_points.is_finite() || track_length_logical_points <= 0.0 {
        return None;
    }
    if !viewport_extent_logical_points.is_finite() || viewport_extent_logical_points <= 0.0 {
        return None;
    }
    let overflow_logical_points = max_offset_subpixels as f64
        / worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    let viewport = f64::from(viewport_extent_logical_points);
    let content = viewport + overflow_logical_points;
    let proportional = f64::from(track_length_logical_points) * viewport / content;
    let length = proportional
        .max(f64::from(metrics.minimum_thumb_length_logical_points()))
        .min(f64::from(track_length_logical_points)) as f32;
    Some(UiScrollThumbExtent {
        length_logical_points: length,
        travel_logical_points: (track_length_logical_points - length).max(0.0),
    })
}

impl UiScrollThumbExtent {
    #[cfg(test)]
    pub(crate) const fn length_logical_points(self) -> f32 {
        self.length_logical_points
    }

    #[cfg(test)]
    pub(crate) const fn travel_logical_points(self) -> f32 {
        self.travel_logical_points
    }

    /// Thumb start relative to the track origin, from the accepted displayed
    /// offset. The bound is the same integer subpixel authority Scroll holds.
    pub(crate) fn start_logical_points(
        self,
        offset_subpixels: i64,
        max_offset_subpixels: i64,
    ) -> f32 {
        if max_offset_subpixels <= 0 {
            return 0.0;
        }
        let clamped = offset_subpixels.clamp(0, max_offset_subpixels);
        let progress = clamped as f64 / max_offset_subpixels as f64;
        (f64::from(self.travel_logical_points) * progress) as f32
    }

    /// The offset a thumb start maps back to, inverting [`Self::start_logical_points`].
    /// A fully compressed travel pins the offset at the origin, because such a
    /// thumb has no position left to express.
    pub(crate) fn offset_subpixels_for_start(
        self,
        start_logical_points: f32,
        max_offset_subpixels: i64,
    ) -> i64 {
        if max_offset_subpixels <= 0 || self.travel_logical_points <= 0.0 {
            return 0;
        }
        let clamped = start_logical_points.clamp(0.0, self.travel_logical_points);
        let progress = f64::from(clamped) / f64::from(self.travel_logical_points);
        (max_offset_subpixels as f64 * progress).round() as i64
    }
}

/// The thumb rectangle, centered across the gutter and placed along the track.
pub(crate) fn thumb_rect(
    axis: super::UiScrollChromeAxis,
    track: worth_ui_host_contract::UiMountedCanonicalBox,
    extent: UiScrollThumbExtent,
    start_logical_points: f32,
    metrics: super::UiScrollChromeMetrics,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    let inset = metrics.thumb_centering_inset_logical_points();
    let thickness = metrics.thumb_thickness_logical_points();
    let input = match axis {
        super::UiScrollChromeAxis::Block => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: track.x() + inset,
            y: track.y() + start_logical_points,
            width: thickness,
            height: extent.length_logical_points,
            coordinate_space: track.coordinate_space(),
        },
        super::UiScrollChromeAxis::Inline => worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: track.x() + start_logical_points,
            y: track.y() + inset,
            width: extent.length_logical_points,
            height: thickness,
            coordinate_space: track.coordinate_space(),
        },
    };
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(input).ok()
}
