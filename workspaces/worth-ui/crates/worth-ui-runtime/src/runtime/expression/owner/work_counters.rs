/// Deterministic operation counts. Invalidation is bounded by these counts,
/// never by timing. They count from session activation; following a new
/// generation never resets them. The work of preparing a generation
/// succession counts when it is prepared, whether or not it commits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiExpressionWorkCounters {
    /// Kernel evaluations run.
    pub evaluations: u64,
    /// Expressions settled by a non-current operand without a kernel run.
    pub settled_without_evaluation: u64,
    /// Owner reads made: one per operand read when an evaluation begins, one
    /// per indexed projection slot examined (at activation, at each published
    /// frame and when a generation is followed), and one per operand
    /// re-proven when a completion is admitted or a record is re-stamped for
    /// a successor generation.
    pub operand_probes: u64,
    /// Readers found through the dependency index.
    pub index_hits: u64,
    /// Completions that changed an outcome and bumped its revision.
    pub published_changes: u64,
    /// Completions whose outcome equaled the retained one.
    pub suppressed_unchanged: u64,
    /// Completions refused: because they began in another active session
    /// (wrong world), in another generation of this session, because the
    /// owner has not followed the active generation, because an operand fact
    /// they read is no longer what its owner holds, or because they name a
    /// slot the retained records do not hold.
    pub stale_completions: u64,
}

impl UiExpressionWorkCounters {
    /// These counts with `work` done besides them added.
    pub(super) const fn absorbing(self, work: Self) -> Self {
        Self {
            evaluations: self.evaluations.saturating_add(work.evaluations),
            settled_without_evaluation: self
                .settled_without_evaluation
                .saturating_add(work.settled_without_evaluation),
            operand_probes: self.operand_probes.saturating_add(work.operand_probes),
            index_hits: self.index_hits.saturating_add(work.index_hits),
            published_changes: self
                .published_changes
                .saturating_add(work.published_changes),
            suppressed_unchanged: self
                .suppressed_unchanged
                .saturating_add(work.suppressed_unchanged),
            stale_completions: self
                .stale_completions
                .saturating_add(work.stale_completions),
        }
    }
}
