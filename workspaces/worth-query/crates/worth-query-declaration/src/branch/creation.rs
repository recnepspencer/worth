/// Caller-chosen posture for one component of a new product branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorthQueryProductBranchComponentPosture {
    ReuseExact,
    Fork,
}

/// Complete authored component posture for one product-branch fork.
///
/// The initial value selects nothing. Each method chooses one component, and
/// runtime admission rejects an incomplete pair before owner work begins.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorthQueryProductBranchComponents {
    relational: Option<WorthQueryProductBranchComponentPosture>,
    signal: Option<WorthQueryProductBranchComponentPosture>,
}

impl WorthQueryProductBranchComponents {
    pub const fn new() -> Self {
        Self {
            relational: None,
            signal: None,
        }
    }

    pub const fn reuse_exact_relational_basis(mut self) -> Self {
        self.relational = Some(WorthQueryProductBranchComponentPosture::ReuseExact);
        self
    }

    pub const fn fork_relational(mut self) -> Self {
        self.relational = Some(WorthQueryProductBranchComponentPosture::Fork);
        self
    }

    pub const fn reuse_exact_signal_basis(mut self) -> Self {
        self.signal = Some(WorthQueryProductBranchComponentPosture::ReuseExact);
        self
    }

    pub const fn fork_signal(mut self) -> Self {
        self.signal = Some(WorthQueryProductBranchComponentPosture::Fork);
        self
    }

    pub const fn relational(self) -> Option<WorthQueryProductBranchComponentPosture> {
        self.relational
    }

    pub const fn signal(self) -> Option<WorthQueryProductBranchComponentPosture> {
        self.signal
    }
}
