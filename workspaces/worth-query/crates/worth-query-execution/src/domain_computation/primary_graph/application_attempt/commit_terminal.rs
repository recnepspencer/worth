use worth_relational::facade::history::BranchId;

/// How a committed receipt became observable to this caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCommitTerminalKind {
    /// This attempt ran the commit and observed its landing directly.
    Executed,
    /// The receipt was rebuilt from the committed record rather than from this
    /// attempt's own execution, for example on an idempotent replay.
    Recovered,
}

/// Opaque, non-authoritative description of how an application commit became
/// observable to this caller.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationCommitTerminalEvidence,
///     WorthQueryApplicationCommitTerminalKind,
/// };
/// use worth_relational::facade::history::BranchId;
///
/// let forged = WorthQueryApplicationCommitTerminalEvidence {
///     kind: WorthQueryApplicationCommitTerminalKind::Recovered,
///     branch: BranchId("forged".to_owned()),
///     attempt_resources_released: None,
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryApplicationCommitTerminalEvidence {
    kind: WorthQueryApplicationCommitTerminalKind,
    branch: BranchId,
    attempt_resources_released: Option<bool>,
}

impl Eq for WorthQueryApplicationCommitTerminalEvidence {}

impl WorthQueryApplicationCommitTerminalEvidence {
    pub(super) fn executed(branch: BranchId, attempt_resources_released: bool) -> Self {
        Self {
            kind: WorthQueryApplicationCommitTerminalKind::Executed,
            branch,
            attempt_resources_released: Some(attempt_resources_released),
        }
    }

    pub(super) const fn recovered(branch: BranchId) -> Self {
        Self {
            kind: WorthQueryApplicationCommitTerminalKind::Recovered,
            branch,
            attempt_resources_released: None,
        }
    }

    pub(super) fn with_retry_cleanup(mut self, attempt_resources_released: bool) -> Self {
        debug_assert_eq!(
            self.kind,
            WorthQueryApplicationCommitTerminalKind::Recovered
        );
        self.attempt_resources_released = Some(attempt_resources_released);
        self
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
