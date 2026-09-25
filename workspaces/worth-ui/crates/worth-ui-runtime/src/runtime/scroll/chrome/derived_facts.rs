//! `UiScrollChromeFacts`: the per-owner bundle of derived scroll chrome.
//!
//! Every field here is reproducible from the stationary viewport box, the
//! Scroll bounds, the accepted displayed offset and the admitted chrome.
//! Destroying the bundle loses nothing, and holding it grants nothing: it
//! carries no authority over offsets, capture or appearance.

/// One axis's derived chrome. Absent from the bundle when that axis has no
/// overflow, so a no-overflow axis has no thumb and no drag target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollChromeAxisFacts {
    track: worth_ui_host_contract::UiMountedCanonicalBox,
    thumb: worth_ui_host_contract::UiMountedCanonicalBox,
    extent: super::UiScrollThumbExtent,
    viewport_extent_logical_points: f32,
    max_offset_subpixels: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollChromeFacts {
    inline: Option<UiScrollChromeAxisFacts>,
    block: Option<UiScrollChromeAxisFacts>,
}

impl UiScrollChromeFacts {
    /// `None` when neither enabled axis overflows, so no chrome exists at all.
    /// Bounds carry the overflow authority Scroll already derived from the
    /// mounted content and viewport boxes; the viewport box supplies placement
    /// and the viewport extent of the thumb proportion.
    pub(crate) fn derive(
        viewport: worth_ui_host_contract::UiMountedCanonicalBox,
        bounds: super::super::UiScrollBounds,
        offset: super::super::UiScrollOffset,
        chrome: &super::UiScrollAdmittedChrome,
    ) -> Option<Self> {
        let inline_overflows = chrome.axes().supports_inline() && bounds.max_inline_subpixels() > 0;
        let block_overflows = chrome.axes().supports_block() && bounds.max_block_subpixels() > 0;
        let corner = if inline_overflows && block_overflows {
            super::UiScrollChromeCorner::SharedWithOtherAxis
        } else {
            super::UiScrollChromeCorner::Unshared
        };
        let inline = inline_overflows
            .then(|| {
                UiScrollChromeAxisFacts::derive(
                    super::UiScrollChromeAxis::Inline,
                    viewport,
                    bounds.max_inline_subpixels(),
                    offset.inline_subpixels(),
                    chrome.metrics(),
                    corner,
                )
            })
            .flatten();
        let block = block_overflows
            .then(|| {
                UiScrollChromeAxisFacts::derive(
                    super::UiScrollChromeAxis::Block,
                    viewport,
                    bounds.max_block_subpixels(),
                    offset.block_subpixels(),
                    chrome.metrics(),
                    corner,
                )
            })
            .flatten();
        (inline.is_some() || block.is_some()).then_some(Self { inline, block })
    }

    pub(crate) const fn axis(
        &self,
        axis: super::UiScrollChromeAxis,
    ) -> Option<UiScrollChromeAxisFacts> {
        match axis {
            super::UiScrollChromeAxis::Inline => self.inline,
            super::UiScrollChromeAxis::Block => self.block,
        }
    }

    /// The square neither track kept, present only while both axes present
    /// chrome. A region with one scrollbar reserves no corner.
    pub(crate) fn corner(&self) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        super::corner_rect(self.inline?.track, self.block?.track)
    }

    /// Which scrollbar a point belongs to. The shared corner belongs to
    /// neither, so this never guesses between two tracks.
    pub(crate) fn pointer_axis(
        &self,
        point: crate::mounting::presentation::UiPlatformPoint,
    ) -> Option<super::UiScrollChromeAxis> {
        [
            (super::UiScrollChromeAxis::Block, self.block),
            (super::UiScrollChromeAxis::Inline, self.inline),
        ]
        .into_iter()
        .find_map(|(axis, facts)| {
            facts
                .filter(|facts| super::rect_contains(facts.track, point))
                .map(|_| axis)
        })
    }

    /// The offset a direct thumb drag places, preserving the grab offset the
    /// press established.
    pub(crate) fn offset_for_thumb_position(
        &self,
        axis: super::UiScrollChromeAxis,
        point: crate::mounting::presentation::UiPlatformPoint,
        grab_offset_logical_points: f32,
        offset: super::super::UiScrollOffset,
    ) -> Option<super::super::UiScrollOffset> {
        let facts = self.axis(axis)?;
        let placed = super::offset_for_thumb_position(
            axis,
            facts.track,
            facts.extent,
            point,
            grab_offset_logical_points,
            facts.max_offset_subpixels,
        );
        facts.offset_with_axis_replaced(axis, placed, offset)
    }

    /// The offset a track click pages to, by one viewport minus one line toward
    /// the pointer. `None` when the point landed on the thumb.
    pub(crate) fn offset_for_track_click(
        &self,
        axis: super::UiScrollChromeAxis,
        point: crate::mounting::presentation::UiPlatformPoint,
        offset: super::super::UiScrollOffset,
        line_extent_logical_points: u16,
    ) -> Option<super::super::UiScrollOffset> {
        let facts = self.axis(axis)?;
        let current = facts.axis_offset_subpixels(axis, offset);
        let paged = super::offset_for_track_click(
            axis,
            facts.thumb,
            point,
            current,
            super::page_step_subpixels(
                facts.viewport_extent_logical_points,
                line_extent_logical_points,
            ),
            facts.max_offset_subpixels,
        )?;
        facts.offset_with_axis_replaced(axis, paged, offset)
    }
}

impl UiScrollChromeAxisFacts {
    fn derive(
        axis: super::UiScrollChromeAxis,
        viewport: worth_ui_host_contract::UiMountedCanonicalBox,
        max_offset_subpixels: i64,
        offset_subpixels: i64,
        metrics: super::UiScrollChromeMetrics,
        corner: super::UiScrollChromeCorner,
    ) -> Option<Self> {
        let track = super::track_rect(axis, viewport, metrics, corner)?;
        let viewport_extent_logical_points = super::viewport_extent_logical_points(axis, viewport);
        let extent = super::thumb_extent(
            super::track_length_logical_points(axis, track),
            viewport_extent_logical_points,
            max_offset_subpixels,
            metrics,
        )?;
        let thumb = super::thumb_rect(
            axis,
            track,
            extent,
            extent.start_logical_points(offset_subpixels, max_offset_subpixels),
            metrics,
        )?;
        Some(Self {
            track,
            thumb,
            extent,
            viewport_extent_logical_points,
            max_offset_subpixels,
        })
    }

    pub(crate) const fn track(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.track
    }

    pub(crate) const fn thumb(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.thumb
    }

    #[cfg(test)]
    pub(crate) const fn extent(self) -> super::UiScrollThumbExtent {
        self.extent
    }

    #[cfg(test)]
    pub(crate) const fn max_offset_subpixels(self) -> i64 {
        self.max_offset_subpixels
    }

    pub(crate) fn effective_thumb_pointer_rect(
        self,
        axis: super::UiScrollChromeAxis,
    ) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        super::effective_thumb_pointer_rect(axis, self.track, self.thumb)
    }

    const fn axis_offset_subpixels(
        self,
        axis: super::UiScrollChromeAxis,
        offset: super::super::UiScrollOffset,
    ) -> i64 {
        match axis {
            super::UiScrollChromeAxis::Inline => offset.inline_subpixels(),
            super::UiScrollChromeAxis::Block => offset.block_subpixels(),
        }
    }

    fn offset_with_axis_replaced(
        self,
        axis: super::UiScrollChromeAxis,
        placed_subpixels: i64,
        offset: super::super::UiScrollOffset,
    ) -> Option<super::super::UiScrollOffset> {
        match axis {
            super::UiScrollChromeAxis::Inline => {
                super::super::UiScrollOffset::new(placed_subpixels, offset.block_subpixels())
            }
            super::UiScrollChromeAxis::Block => {
                super::super::UiScrollOffset::new(offset.inline_subpixels(), placed_subpixels)
            }
        }
    }
}
