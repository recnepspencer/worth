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
}

impl UiScrollAxes {
    pub(super) const fn accepts_inline(self) -> bool {
        matches!(self, Self::Inline | Self::Both)
    }

    pub(super) const fn accepts_block(self) -> bool {
        matches!(self, Self::Block | Self::Both)
    }
}
