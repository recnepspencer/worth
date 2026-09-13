use worth_store_recovery_physics::PageLsn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCompletion {
    recovered_root: String,
    admitted_page_lsn_frontier: Option<PageLsn>,
    replayed_frames: usize,
    source_candidate_count: usize,
    source_decision_digest: String,
}

#[cfg(feature = "certification-test-authority")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCompletionDenial {
    EmptyRecoveredRoot,
    EmptySourceDecisionDigest,
}

/// Seals the descriptive facts produced by the runtime's terminal recovery
/// boundary. The value carries no Store authority; authority remains in the
/// consuming runtime handoff.
#[cfg(feature = "certification-test-authority")]
pub fn complete_recovery(
    recovered_root: impl Into<String>,
    admitted_page_lsn_frontier: Option<PageLsn>,
    replayed_frames: usize,
    source_candidate_count: usize,
    source_decision_digest: impl Into<String>,
) -> Result<RecoveryCompletion, RecoveryCompletionDenial> {
    let recovered_root = recovered_root.into();
    if recovered_root.is_empty() {
        return Err(RecoveryCompletionDenial::EmptyRecoveredRoot);
    }
    let source_decision_digest = source_decision_digest.into();
    if source_decision_digest.is_empty() {
        return Err(RecoveryCompletionDenial::EmptySourceDecisionDigest);
    }
    Ok(RecoveryCompletion {
        recovered_root,
        admitted_page_lsn_frontier,
        replayed_frames,
        source_candidate_count,
        source_decision_digest,
    })
}

impl RecoveryCompletion {
    pub fn recovered_root(&self) -> &str {
        &self.recovered_root
    }

    pub const fn admitted_page_lsn_frontier(&self) -> Option<PageLsn> {
        self.admitted_page_lsn_frontier
    }

    pub const fn replayed_frames(&self) -> usize {
        self.replayed_frames
    }

    pub const fn source_candidate_count(&self) -> usize {
        self.source_candidate_count
    }

    pub fn source_decision_digest(&self) -> &str {
        &self.source_decision_digest
    }
}
