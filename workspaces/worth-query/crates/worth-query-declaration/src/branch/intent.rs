use super::WorthQueryProductBranchComponents;

/// Pure authored meaning for creating one product branch from an explicitly
/// selected source occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorthQueryProductBranchForkIntent {
    components: WorthQueryProductBranchComponents,
}

impl WorthQueryProductBranchForkIntent {
    pub const fn new(components: WorthQueryProductBranchComponents) -> Self {
        Self { components }
    }

    pub const fn components(self) -> WorthQueryProductBranchComponents {
        self.components
    }
}
