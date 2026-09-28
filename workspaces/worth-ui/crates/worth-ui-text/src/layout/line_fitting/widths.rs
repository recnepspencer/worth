//! The widths a fit decides alike at, and the widths a layout stands for.

/// The widths a fit compared against its maximum width: every width it kept
/// within the maximum, and the least width it wrapped at. Any maximum from
/// `minimum` up to, not including, `limit` makes each comparison come out
/// the same way, so it fits the same lines.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::layout) struct FitWidths {
    minimum: i64,
    limit: Option<i64>,
}

/// The widths a layout was fitted for, from `minimum` up to, not including,
/// `limit`. Fitting at any of them lays the paragraph out identically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::layout) struct ReflowWidths {
    minimum: u32,
    limit: Option<u32>,
}

impl FitWidths {
    pub(in crate::layout) fn hold(&mut self, width: i64) {
        self.minimum = self.minimum.max(width);
    }

    pub(super) fn wrap_at(&mut self, width: i64) {
        self.limit = Some(self.limit.map_or(width, |limit| limit.min(width)));
    }

    /// The widths both fits decide alike at.
    pub(in crate::layout) fn meet(self, other: Self) -> Self {
        let mut met = self;
        met.hold(other.minimum);
        if let Some(limit) = other.limit {
            met.wrap_at(limit);
        }
        met
    }

    /// The widths a layout fitted at `requested` stands for: only `requested`
    /// itself unless these comparisons were all the layout read of its width.
    pub(in crate::layout) fn reflow(
        self,
        decided_by_fit_alone: bool,
        requested: u32,
    ) -> ReflowWidths {
        let exact = ReflowWidths {
            minimum: requested,
            limit: requested.checked_add(1),
        };
        if !decided_by_fit_alone {
            return exact;
        }
        let Ok(minimum) = u32::try_from(self.minimum.max(1)) else {
            return exact;
        };
        let reflow = ReflowWidths {
            minimum,
            limit: self.limit.and_then(|limit| u32::try_from(limit).ok()),
        };
        debug_assert!(reflow.admits(requested), "a fit admits its own width");
        if reflow.admits(requested) {
            reflow
        } else {
            exact
        }
    }
}

impl ReflowWidths {
    /// The least width the layout stands for, which names it.
    pub(in crate::layout) const fn fitted_width(self) -> u32 {
        self.minimum
    }

    pub(in crate::layout) fn admits(self, width_millipoints: u32) -> bool {
        width_millipoints >= self.minimum
            && self.limit.is_none_or(|limit| width_millipoints < limit)
    }
}
