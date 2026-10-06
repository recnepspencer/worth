use worth_relational::facade::history::CommitId;

/// An admitted observation of an earlier commit with the same durable intent.
/// It grants no mutation, publication, workflow settlement, or receipt authority.
/// Read current state through a freshly admitted query; the original handler
/// result and live commit receipt are not retained by this observation.
///
/// A caller may inspect the fact but cannot manufacture it from a commit id.
///
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryHistoricalApplicationCommit;
/// fn inspect(observed: &WorthQueryHistoricalApplicationCommit) {
///     let _ = observed.commit_id();
/// }
/// ```
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryHistoricalApplicationCommit;
/// use worth_relational::facade::history::CommitId;
/// let forged = WorthQueryHistoricalApplicationCommit { commit_id: CommitId(1) };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryHistoricalApplicationCommit {
    commit_id: CommitId,
}

impl WorthQueryHistoricalApplicationCommit {
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub(super) const fn observed(commit_id: CommitId) -> Self {
        Self { commit_id }
    }
}
