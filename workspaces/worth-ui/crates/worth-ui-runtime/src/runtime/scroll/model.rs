#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollDelta {
    inline_subpixels: i64,
    block_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiScrollOffset {
    inline_subpixels: i64,
    block_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollBounds {
    max_inline_subpixels: i64,
    max_block_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollAxes {
    Inline,
    Block,
    Both,
}

impl UiScrollDelta {
    pub(crate) const fn new(inline_subpixels: i64, block_subpixels: i64) -> Self {
        Self {
            inline_subpixels,
            block_subpixels,
        }
    }

    pub(crate) const fn inline_subpixels(self) -> i64 {
        self.inline_subpixels
    }

    pub(crate) const fn block_subpixels(self) -> i64 {
        self.block_subpixels
    }

    pub(super) const fn is_zero(self) -> bool {
        self.inline_subpixels == 0 && self.block_subpixels == 0
    }

    pub(super) fn subtract(self, consumed: Self) -> Self {
        Self::new(
            self.inline_subpixels - consumed.inline_subpixels,
            self.block_subpixels - consumed.block_subpixels,
        )
    }
}

impl UiScrollOffset {
    pub(crate) const fn new(inline_subpixels: i64, block_subpixels: i64) -> Option<Self> {
        if inline_subpixels < 0 || block_subpixels < 0 {
            None
        } else {
            Some(Self {
                inline_subpixels,
                block_subpixels,
            })
        }
    }

    pub(crate) const fn origin() -> Self {
        Self {
            inline_subpixels: 0,
            block_subpixels: 0,
        }
    }

    pub(crate) const fn inline_subpixels(self) -> i64 {
        self.inline_subpixels
    }

    pub(crate) const fn block_subpixels(self) -> i64 {
        self.block_subpixels
    }
}

impl UiScrollBounds {
    pub(crate) fn axes(self) -> UiScrollAxes {
        match (self.max_inline_subpixels > 0, self.max_block_subpixels > 0) {
            (true, false) => UiScrollAxes::Inline,
            (false, true) => UiScrollAxes::Block,
            (true, true) | (false, false) => UiScrollAxes::Both,
        }
    }

    pub(crate) fn from_mounted_region(
        content: worth_ui_host_contract::UiMountedCanonicalBox,
        viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> Option<Self> {
        let extent = |content: f32, viewport: f32| {
            let value = f64::from((content - viewport).max(0.0))
                * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
            (value.is_finite() && value < i64::MAX as f64).then(|| value.round() as i64)
        };
        Self::new(
            extent(content.width(), viewport.width())?,
            extent(content.height(), viewport.height())?,
        )
    }

    pub(crate) const fn new(max_inline_subpixels: i64, max_block_subpixels: i64) -> Option<Self> {
        if max_inline_subpixels < 0 || max_block_subpixels < 0 {
            None
        } else {
            Some(Self {
                max_inline_subpixels,
                max_block_subpixels,
            })
        }
    }

    pub(crate) const fn max_inline_subpixels(self) -> i64 {
        self.max_inline_subpixels
    }

    pub(crate) const fn max_block_subpixels(self) -> i64 {
        self.max_block_subpixels
    }

    /// Whether these bounds leave anywhere to scroll to. Content that emptied
    /// and content that shrank to fit its viewport both land here, and neither
    /// has a second offset to reach, so a settle under them is traveling
    /// toward the only place it can already be.
    pub(crate) const fn admits_no_travel(self) -> bool {
        self.max_inline_subpixels == 0 && self.max_block_subpixels == 0
    }

    pub(super) const fn contains(self, offset: UiScrollOffset) -> bool {
        offset.inline_subpixels <= self.max_inline_subpixels
            && offset.block_subpixels <= self.max_block_subpixels
    }

    pub(super) const fn clamp(self, offset: UiScrollOffset) -> UiScrollOffset {
        UiScrollOffset {
            inline_subpixels: if offset.inline_subpixels > self.max_inline_subpixels {
                self.max_inline_subpixels
            } else {
                offset.inline_subpixels
            },
            block_subpixels: if offset.block_subpixels > self.max_block_subpixels {
                self.max_block_subpixels
            } else {
                offset.block_subpixels
            },
        }
    }

    /// The offset nearest `[inline, block]` that these bounds admit, for a pair
    /// of axis values that has not been proven non-negative.
    ///
    /// An offset is a distance traveled from rest, so a negative one names a
    /// place behind rest and lands at rest. Alignment arithmetic reaches that
    /// place routinely -- revealing something already at the top asks to scroll
    /// past the beginning -- and answering it here is what lets the reveal lane
    /// construct an offset without asserting a range it did not check.
    pub(crate) const fn clamp_subpixels(
        self,
        inline_subpixels: i64,
        block_subpixels: i64,
    ) -> UiScrollOffset {
        UiScrollOffset {
            inline_subpixels: clamp_axis(inline_subpixels, self.max_inline_subpixels),
            block_subpixels: clamp_axis(block_subpixels, self.max_block_subpixels),
        }
    }
}

/// One axis of `clamp_subpixels`: rest is the floor, the declared extent the
/// ceiling, and a ceiling below rest collapses to rest.
const fn clamp_axis(value: i64, max: i64) -> i64 {
    if value < 0 {
        0
    } else if value > max {
        max
    } else {
        value
    }
}

impl UiScrollAxes {
    pub(super) const fn accepts_inline(self) -> bool {
        matches!(self, Self::Inline | Self::Both)
    }

    pub(super) const fn accepts_block(self) -> bool {
        matches!(self, Self::Block | Self::Both)
    }
}
