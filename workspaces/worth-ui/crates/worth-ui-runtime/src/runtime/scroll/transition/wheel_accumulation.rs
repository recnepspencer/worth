//! Coarse-wheel evidence becoming scroll points, and the accumulation window
//! that says whether a burst is still in progress.
//!
//! The host reports notches already multiplied by the platform lines-per-notch
//! count, encoded as thousandths of a line so a high-resolution wheel keeps its
//! fraction. The runtime supplies the region's declared line extent and forms
//! the product: one notch of three lines against a 20-point line extent is
//! 60 points, never a device-layer constant.

/// Thousandths of a line per reported line, matching the host's line encoding.
pub(crate) const UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE: i64 = 1_000;

/// A host-reported coarse-wheel delta in thousandths of a line per axis.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiScrollWheelLineDelta {
    inline_milli_lines: i64,
    block_milli_lines: i64,
}

/// The accumulation window for one Scroll owner. `Started` opens it, `Updated`
/// extends it, `Ended` and `Cancelled` close it. An unphased coarse wheel event
/// arrives as `Updated` with no window open and opens one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollWheelWindow {
    open: bool,
    latest_input_tick: u64,
}

/// One admitted coarse-wheel event: its line delta, its phase, the tick it was
/// observed at, and the declared extents that turn it into points and a horizon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollWheelInput {
    lines: UiScrollWheelLineDelta,
    phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
    input_tick: u64,
    line_extent_logical_points: u16,
    settle_ticks: u32,
}

impl UiScrollWheelLineDelta {
    pub(crate) const fn new(inline_milli_lines: i64, block_milli_lines: i64) -> Self {
        Self {
            inline_milli_lines,
            block_milli_lines,
        }
    }

    /// Whole notches of `lines_per_notch` lines each, the ordinary coarse-wheel
    /// authoring shape.
    #[cfg(test)]
    pub(crate) const fn from_notches(
        inline_notches: i64,
        block_notches: i64,
        lines_per_notch: u16,
    ) -> Self {
        let per_notch = UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE * lines_per_notch as i64;
        Self {
            inline_milli_lines: inline_notches * per_notch,
            block_milli_lines: block_notches * per_notch,
        }
    }

    #[cfg(test)]
    pub(crate) const fn inline_milli_lines(self) -> i64 {
        self.inline_milli_lines
    }

    #[cfg(test)]
    pub(crate) const fn block_milli_lines(self) -> i64 {
        self.block_milli_lines
    }
}

impl UiScrollWheelWindow {
    pub(crate) const fn closed() -> Self {
        Self {
            open: false,
            latest_input_tick: 0,
        }
    }

    #[cfg(test)]
    pub(crate) const fn is_open(self) -> bool {
        self.open
    }

    #[cfg(test)]
    pub(crate) const fn latest_input_tick(self) -> u64 {
        self.latest_input_tick
    }

    pub(crate) const fn observe(
        self,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        input_tick: u64,
    ) -> Self {
        Self {
            open: match phase {
                worth_ui_host_contract::UiHostScrollDeltaPhase::Started
                | worth_ui_host_contract::UiHostScrollDeltaPhase::Updated => true,
                worth_ui_host_contract::UiHostScrollDeltaPhase::Ended
                | worth_ui_host_contract::UiHostScrollDeltaPhase::Cancelled => false,
            },
            latest_input_tick: input_tick,
        }
    }
}

impl UiScrollWheelInput {
    pub(crate) const fn admit(
        lines: UiScrollWheelLineDelta,
        phase: worth_ui_host_contract::UiHostScrollDeltaPhase,
        input_tick: u64,
        line_extent_logical_points: u16,
        settle_ticks: u32,
    ) -> Result<Self, super::UiScrollTransitionDenial> {
        if line_extent_logical_points == 0 {
            return Err(super::UiScrollTransitionDenial::LineExtentUnusable);
        }
        if settle_ticks == 0 {
            return Err(super::UiScrollTransitionDenial::SettleHorizonUnavailable);
        }
        Ok(Self {
            lines,
            phase,
            input_tick,
            line_extent_logical_points,
            settle_ticks,
        })
    }

    /// Lines times the declared line extent, in the same integer subpixels the
    /// Scroll offset model uses. A thousandth of a line against a 20-point
    /// extent is exactly 20 subpixels, so fractions survive the product.
    pub(crate) fn points_subpixels(
        self,
    ) -> Result<super::super::UiScrollDelta, super::UiScrollTransitionDenial> {
        let extent = i64::from(self.line_extent_logical_points);
        let axis = |milli_lines: i64| {
            milli_lines
                .checked_mul(extent)
                .ok_or(super::UiScrollTransitionDenial::ArithmeticOutOfRange)
        };
        Ok(super::super::UiScrollDelta::new(
            axis(self.lines.inline_milli_lines)?,
            axis(self.lines.block_milli_lines)?,
        ))
    }

    pub(crate) const fn phase(self) -> worth_ui_host_contract::UiHostScrollDeltaPhase {
        self.phase
    }

    pub(crate) const fn input_tick(self) -> u64 {
        self.input_tick
    }

    pub(crate) const fn settle_ticks(self) -> u32 {
        self.settle_ticks
    }
}
