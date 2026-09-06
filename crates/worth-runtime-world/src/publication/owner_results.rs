use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::branch::RelationalForkOutcome;
use worth_relational::facade::history::RelationalCommitIdentity;
use worth_relational::facade::publication::DeferredPublicationSettlement;
use worth_relational::facade::transactions::CommitResult;

use crate::history::CompositeComponentChangePosture;

#[path = "owner_results/signal.rs"]
mod signal;
pub use signal::CompositeSignalOwnerResult;

#[path = "owner_results/relational.rs"]
mod relational;

/// Exact result of the Relational leg. The retained variant is evidence that
/// no Relational owner movement was requested, not an absent or guessed result.
#[derive(Debug)]
pub struct CompositeRelationalOwnerResult {
    result: CompositeRelationalOwnerResultKind,
}

#[derive(Debug, Clone)]
enum CompositeRelationalOwnerResultKind {
    RetainedExact,
    Published {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        settlement: Option<worth_relational::facade::history::RelationalCommitReceipt>,
        result: Option<std::sync::Arc<CommitResult>>,
    },
    Forked {
        fork: RelationalForkOutcome,
        successor_basis: AdmittedRelationalBranchBasis,
    },
    SettlementRequired {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
    },
    SettlementPending {
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        _settlement: DeferredPublicationSettlement,
    },
}

/// The two owner results carried by one performed publication. They are
/// created only from the corresponding owner progress and cannot be mixed
/// independently with a commit posture.
#[derive(Debug)]
pub struct CompositeOwnerExecutionResults {
    relational: CompositeRelationalOwnerResult,
    signal: CompositeSignalOwnerResult,
}

impl CompositeOwnerExecutionResults {
    /// Share immutable owner-issued evidence without duplicating a phase or
    /// performed authority. The public result remains non-Clone.
    pub(crate) fn evidence_image(&self) -> Self {
        Self {
            relational: CompositeRelationalOwnerResult {
                result: self.relational.result.clone(),
            },
            signal: self.signal.evidence_image(),
        }
    }

    pub(super) fn from_components(
        relational: CompositeRelationalOwnerResult,
        signal: CompositeSignalOwnerResult,
    ) -> Self {
        Self { relational, signal }
    }

    pub(crate) fn retained() -> Self {
        Self {
            relational: CompositeRelationalOwnerResult::retained(),
            signal: CompositeSignalOwnerResult::retained(),
        }
    }

    pub(crate) fn with_relational_settled(
        self,
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        result: CommitResult,
    ) -> Self {
        let (_relational, signal) = self.into_parts();
        Self {
            relational: CompositeRelationalOwnerResult::settled(
                commit_identity,
                successor_basis,
                std::sync::Arc::new(result),
            ),
            signal,
        }
    }

    pub(crate) fn with_relational_settled_receipt(
        self,
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        receipt: worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Self {
        let (_relational, signal) = self.into_parts();
        Self {
            relational: CompositeRelationalOwnerResult::settled_receipt(
                commit_identity,
                successor_basis,
                receipt,
            ),
            signal,
        }
    }

    pub(crate) fn with_relational_settlement_pending(
        self,
        commit_identity: RelationalCommitIdentity,
        successor_basis: AdmittedRelationalBranchBasis,
        settlement: DeferredPublicationSettlement,
    ) -> Self {
        let (_relational, signal) = self.into_parts();
        Self {
            relational: CompositeRelationalOwnerResult::settlement_pending(
                commit_identity,
                successor_basis,
                settlement,
            ),
            signal,
        }
    }

    fn into_parts(self) -> (CompositeRelationalOwnerResult, CompositeSignalOwnerResult) {
        (self.relational, self.signal)
    }

    pub fn relational_posture(&self) -> CompositeComponentChangePosture {
        match self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact => {
                CompositeComponentChangePosture::RetainExact
            }
            CompositeRelationalOwnerResultKind::Published { .. }
            | CompositeRelationalOwnerResultKind::Forked { .. }
            | CompositeRelationalOwnerResultKind::SettlementPending { .. }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. } => {
                CompositeComponentChangePosture::Published
            }
        }
    }

    pub fn signal(&self) -> &CompositeSignalOwnerResult {
        &self.signal
    }

    pub fn signal_posture(&self) -> CompositeComponentChangePosture {
        self.signal.posture()
    }

    pub fn relational_publication_identity(
        &self,
    ) -> Option<worth_relational::facade::history::RelationalCommitIdentity> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact => None,
            CompositeRelationalOwnerResultKind::Published {
                commit_identity, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementPending {
                commit_identity, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementRequired {
                commit_identity, ..
            } => Some(commit_identity.clone()),
            CompositeRelationalOwnerResultKind::Forked { .. } => None,
        }
    }

    pub fn relational_publication_basis_identity(
        &self,
    ) -> Option<&worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact => None,
            CompositeRelationalOwnerResultKind::Published {
                successor_basis, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementPending {
                successor_basis, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementRequired {
                successor_basis, ..
            }
            | CompositeRelationalOwnerResultKind::Forked {
                successor_basis, ..
            } => Some(successor_basis.admission_identity()),
        }
    }

    pub fn relational_fork_target_identity(
        &self,
    ) -> Option<&worth_relational::facade::branch::RelationalBranchIdentity> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::Forked { fork, .. } => Some(fork.target_identity()),
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Published { .. }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. }
            | CompositeRelationalOwnerResultKind::SettlementPending { .. } => None,
        }
    }

    pub fn relational_settlement(
        &self,
    ) -> Option<&worth_relational::facade::history::RelationalCommitReceipt> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::Published { settlement, .. } => settlement.as_ref(),
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Forked { .. }
            | CompositeRelationalOwnerResultKind::SettlementPending { .. }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. } => None,
        }
    }

    pub fn relational_commit_result(&self) -> Option<&CommitResult> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::Published { result, .. } => result.as_deref(),
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Forked { .. }
            | CompositeRelationalOwnerResultKind::SettlementPending { .. }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. } => None,
        }
    }

    pub fn signal_publication_identity(
        &self,
    ) -> Option<crate::history::CompositeSignalPublicationIdentity> {
        self.signal.publication_identity()
    }

    pub(crate) fn matches_plan(&self, plan: &super::LoweredOwnerComponentPlan) -> bool {
        use super::RelationalComponentPlanPosture;

        let relational_matches = match plan.relational().posture() {
            RelationalComponentPlanPosture::RetainExact => {
                self.relational_posture() == CompositeComponentChangePosture::RetainExact
            }
            RelationalComponentPlanPosture::PublishPrepared => {
                matches!(
                    self.relational.result,
                    CompositeRelationalOwnerResultKind::Published { .. }
                        | CompositeRelationalOwnerResultKind::SettlementRequired { .. }
                        | CompositeRelationalOwnerResultKind::SettlementPending { .. }
                )
            }
        };
        let signal_matches = self.signal.matches_plan(plan.signal().posture());
        relational_matches && signal_matches
    }

    /// Whether both owner results are the exact evidence the creation plan
    /// asked each owner to produce.
    pub(crate) fn matches_creation_plan(
        &self,
        plan: &crate::branch::LoweredBranchCreationPlan,
    ) -> bool {
        let relational_matches = match plan.relational() {
            crate::branch::RelationalBranchCreationPlan::ReuseExact => matches!(
                self.relational.result,
                CompositeRelationalOwnerResultKind::RetainedExact
            ),
            crate::branch::RelationalBranchCreationPlan::ForkExact { .. } => matches!(
                self.relational.result,
                CompositeRelationalOwnerResultKind::Forked { .. }
            ),
        };
        relational_matches && self.signal.matches_creation_plan(plan.signal())
    }

    /// Whether both owner results are honest evidence for a creation that
    /// stopped partway. A sibling denial leaves the later owner untouched, so
    /// a retained leg is admissible here; a leg that did move must still be
    /// exactly the evidence its creation plan asked for.
    pub(crate) fn matches_partial_creation_plan(
        &self,
        plan: &crate::branch::LoweredBranchCreationPlan,
    ) -> bool {
        let relational_matches = matches!(
            self.relational.result,
            CompositeRelationalOwnerResultKind::RetainedExact
        ) || match plan.relational() {
            crate::branch::RelationalBranchCreationPlan::ReuseExact => true,
            crate::branch::RelationalBranchCreationPlan::ForkExact { .. } => matches!(
                self.relational.result,
                CompositeRelationalOwnerResultKind::Forked { .. }
            ),
        };
        relational_matches && self.signal.matches_partial_creation_plan(plan.signal())
    }
}
