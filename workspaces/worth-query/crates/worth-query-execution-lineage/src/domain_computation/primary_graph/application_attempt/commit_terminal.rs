use worth_relational::facade::history::BranchId;

/// How a committed receipt became observable to this caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitTerminalKind {
    /// This attempt ran the commit and observed its landing directly.
    Executed,
}

/// Opaque, non-authoritative description of how an application commit became
/// observable to this caller.
#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryApplicationCommitTerminalEvidence {
    kind: WorthQueryApplicationCommitTerminalKind,
    branch: BranchId,
    attempt_resources_released: Option<bool>,
}

impl Eq for WorthQueryApplicationCommitTerminalEvidence {}

impl WorthQueryApplicationCommitTerminalEvidence {
    pub fn executed(branch: BranchId, attempt_resources_released: bool) -> Self {
        Self {
            kind: WorthQueryApplicationCommitTerminalKind::Executed,
            branch,
            attempt_resources_released: Some(attempt_resources_released),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationCommitTerminalKind {
        self.kind
    }

    pub const fn branch(&self) -> &BranchId {
        &self.branch
    }

    pub const fn attempt_resources_released(&self) -> Option<bool> {
        self.attempt_resources_released
    }
}
