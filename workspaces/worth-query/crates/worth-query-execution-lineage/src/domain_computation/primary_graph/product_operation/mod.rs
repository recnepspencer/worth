//! Pure product-operation stop and selection-origin types.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SelectedQueryBasisRetentionStop {
    Basis,
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
}

/// How the live selected snapshot entered this operation. This is selection
/// provenance, not a promise that its head remains latest during execution.
#[derive(Clone, Copy, Debug, Default)]
pub struct SelectedQueryOrigin {
    current_at_selection: bool,
}

impl SelectedQueryOrigin {
    pub const fn current_at_selection() -> Self {
        Self {
            current_at_selection: true,
        }
    }

    pub const fn is_current(self) -> bool {
        self.current_at_selection
    }
}
