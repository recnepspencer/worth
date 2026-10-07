use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::diagnostics::history::BridgeHistoricalEvaluationCounters;
use crate::identity::{BridgeIdentity, HistoricalEvaluationFailureIdentityTag};
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::{
    BridgeTruthViewSelectorIdentity, HistoricalEvaluationDeclarationIdentity, TruthSnapshotIdentity,
};

pub type BridgeHistoricalEvaluationFailureIdentity =
    BridgeIdentity<HistoricalEvaluationFailureIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeHistoricalEvaluationFailureClass {
    ExecutionDenied(crate::error::BridgeExecutionDenial),
    UnsupportedTruthViewSelector,
    TruthViewUnavailable,
    RejectedBranchMismatch,
    RejectedSnapshotMismatch,
    RejectedHistoricalResolutionFailure,
    HistoricalReplayMismatch,
    UnresolvedTruthViewPolicyConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalEvaluationFailureRecord {
    failure_identity: BridgeHistoricalEvaluationFailureIdentity,
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    selector_identity: BridgeTruthViewSelectorIdentity,
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: Option<TruthSnapshotIdentity>,
    failure_class: BridgeHistoricalEvaluationFailureClass,
    detail: Arc<str>,
    counters: BridgeHistoricalEvaluationCounters,
}

impl BridgeHistoricalEvaluationFailureRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        declaration_identity: HistoricalEvaluationDeclarationIdentity,
        selector_identity: BridgeTruthViewSelectorIdentity,
        branch_identity: TruthBranchIdentity,
        commit_identity: Option<TruthCommitIdentity>,
        snapshot_identity: Option<TruthSnapshotIdentity>,
        failure_class: BridgeHistoricalEvaluationFailureClass,
        detail: impl Into<Arc<str>>,
        counters: BridgeHistoricalEvaluationCounters,
    ) -> Self {
        let detail = detail.into();
        let canonical_basis = format!(
            "historical-evaluation-failure|declaration={}|selector={}|branch={}|commit={}|snapshot={}|class:{failure_class:?}|detail={}",
            declaration_identity.as_str(),
            selector_identity.as_str(),
            branch_identity.as_str(),
            commit_identity
                .as_ref()
                .map(TruthCommitIdentity::as_str)
                .unwrap_or("-"),
            snapshot_identity
                .as_ref()
                .map(TruthSnapshotIdentity::as_str)
                .unwrap_or("-"),
            detail.as_ref(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            failure_identity: BridgeHistoricalEvaluationFailureIdentity::admit_bridge_owned(
                format!("historical-evaluation-failure:sha256:{digest:x}"),
            ),
            declaration_identity,
            selector_identity,
            branch_identity,
            commit_identity,
            snapshot_identity,
            failure_class,
            detail,
            counters,
        }
    }

    pub fn failure_identity(&self) -> &BridgeHistoricalEvaluationFailureIdentity {
        &self.failure_identity
    }

    pub fn declaration_identity(&self) -> &HistoricalEvaluationDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn selector_identity(&self) -> &BridgeTruthViewSelectorIdentity {
        &self.selector_identity
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> Option<&TruthSnapshotIdentity> {
        self.snapshot_identity.as_ref()
    }

    pub fn failure_class(&self) -> BridgeHistoricalEvaluationFailureClass {
        self.failure_class
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }

    pub fn counters(&self) -> &BridgeHistoricalEvaluationCounters {
        &self.counters
    }
}
