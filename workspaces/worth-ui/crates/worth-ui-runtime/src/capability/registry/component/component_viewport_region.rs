use super::ComponentViewportAxisPlacement;

/// Authored component geometry within a reference box: the logical viewport,
/// or the layout cell a container assigns. Each axis declares whether it
/// keeps a fixed extent or absorbs the box's remaining extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentViewportRegion {
    horizontal: ComponentViewportAxisPlacement,
    vertical: ComponentViewportAxisPlacement,
}

impl ComponentViewportRegion {
    pub const fn new(
        horizontal: ComponentViewportAxisPlacement,
        vertical: ComponentViewportAxisPlacement,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub const fn horizontal(self) -> ComponentViewportAxisPlacement {
        self.horizontal
    }

    pub const fn vertical(self) -> ComponentViewportAxisPlacement {
        self.vertical
    }

    pub(crate) fn digest_basis(self) -> String {
        format!(
            "viewport-region:{}:{}",
            self.horizontal.digest_basis(),
            self.vertical.digest_basis(),
        )
    }
}
