pub const MAX_ORDINARY_COLLECTION_WINDOW_WIDTH: u32 = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCollectionWindowBreadth {
    viewport_rows: u32,
    overscan_before: u32,
    overscan_after: u32,
    mounting_budget: u32,
    admitted_width: u32,
}

impl WorthQueryCollectionWindowBreadth {
    pub fn new(
        viewport_rows: u32,
        overscan_before: u32,
        overscan_after: u32,
        mounting_budget: u32,
    ) -> Result<Self, WorthQueryCollectionWindowBreadthDenial> {
        if viewport_rows == 0 {
            return Err(WorthQueryCollectionWindowBreadthDenial::new(
                WorthQueryCollectionWindowBreadthDenialKind::EmptyViewport,
            ));
        }
        if mounting_budget == 0 {
            return Err(WorthQueryCollectionWindowBreadthDenial::new(
                WorthQueryCollectionWindowBreadthDenialKind::EmptyMountingBudget,
            ));
        }
        let requested_width = viewport_rows
            .checked_add(overscan_before)
            .and_then(|width| width.checked_add(overscan_after))
            .ok_or_else(|| {
                WorthQueryCollectionWindowBreadthDenial::new(
                    WorthQueryCollectionWindowBreadthDenialKind::ArithmeticOverflow,
                )
            })?;
        let admitted_width = requested_width.min(mounting_budget);
        if admitted_width > MAX_ORDINARY_COLLECTION_WINDOW_WIDTH {
            return Err(WorthQueryCollectionWindowBreadthDenial::new(
                WorthQueryCollectionWindowBreadthDenialKind::MaximumExceeded,
            ));
        }
        Ok(Self {
            viewport_rows,
            overscan_before,
            overscan_after,
            mounting_budget,
            admitted_width,
        })
    }

    pub const fn viewport_rows(self) -> u32 {
        self.viewport_rows
    }

    pub const fn overscan_before(self) -> u32 {
        self.overscan_before
    }

    pub const fn overscan_after(self) -> u32 {
        self.overscan_after
    }

    pub const fn mounting_budget(self) -> u32 {
        self.mounting_budget
    }

    pub const fn admitted_width(self) -> u32 {
        self.admitted_width
    }

    pub const fn mounting_budget_clamped(self) -> bool {
        self.admitted_width < self.viewport_rows + self.overscan_before + self.overscan_after
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCollectionWindowBreadthDenialKind {
    EmptyViewport,
    EmptyMountingBudget,
    ArithmeticOverflow,
    MaximumExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCollectionWindowBreadthDenial {
    kind: WorthQueryCollectionWindowBreadthDenialKind,
}

impl WorthQueryCollectionWindowBreadthDenial {
    fn new(kind: WorthQueryCollectionWindowBreadthDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> WorthQueryCollectionWindowBreadthDenialKind {
        self.kind
    }
}
#[cfg(test)]
#[path = "breadth_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
