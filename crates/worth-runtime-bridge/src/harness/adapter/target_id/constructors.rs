use super::BridgeHarnessTargetId;
use crate::facade::{TruthBranchIdentity, TruthCommitIdentity};
use crate::source::SourceDeclarationIdentity;

impl BridgeHarnessTargetId {
    pub fn committed_route(commit_identity: TruthCommitIdentity) -> Self {
        Self::CommittedRoute { commit_identity }
    }

    pub fn source_materialize(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceMaterialize {
            declaration_identity,
        }
    }

    pub fn source_materialize_batch(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceMaterializeBatch {
            declaration_identity,
        }
    }

    pub fn source_replay(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceReplay {
            declaration_identity,
        }
    }

    pub fn source_reject_unregistered(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceRejectUnregistered {
            declaration_identity,
        }
    }

    pub fn source_reject_open_snapshot(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceRejectOpenSnapshot {
            declaration_identity,
        }
    }

    pub fn source_reject_snapshot_drift(declaration_identity: SourceDeclarationIdentity) -> Self {
        Self::SourceRejectSnapshotDrift {
            declaration_identity,
        }
    }

    pub fn policy_provenance_certification() -> Self {
        Self::PolicyProvenanceCertification
    }

    pub fn policy_rejection_certification() -> Self {
        Self::PolicyRejectionCertification
    }

    pub fn policy_ambient_leak_certification() -> Self {
        Self::PolicyAmbientLeakCertification
    }

    pub fn speculation_discard_certification() -> Self {
        Self::SpeculationDiscardCertification
    }

    pub fn speculation_promotion_certification() -> Self {
        Self::SpeculationPromotionCertification
    }

    pub fn speculation_churn_certification() -> Self {
        Self::SpeculationChurnCertification
    }

    pub fn historical_commit(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
    ) -> Self {
        Self::HistoricalCommit {
            branch_identity,
            commit_identity,
        }
    }

    pub fn branch_head(branch_identity: TruthBranchIdentity) -> Self {
        Self::BranchHead { branch_identity }
    }
}
