#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryBasisSupport {
    current: bool,
    pinned: bool,
    preview: bool,
}

impl ApplicationQueryBasisSupport {
    pub const fn current_only() -> Self {
        Self {
            current: true,
            pinned: false,
            preview: false,
        }
    }

    pub const fn current_and_pinned() -> Self {
        Self {
            current: true,
            pinned: true,
            preview: false,
        }
    }

    pub const fn with_preview(mut self) -> Self {
        self.preview = true;
        self
    }

    pub const fn current(self) -> bool {
        self.current
    }

    pub const fn pinned(self) -> bool {
        self.pinned
    }

    pub const fn preview(self) -> bool {
        self.preview
    }
}
