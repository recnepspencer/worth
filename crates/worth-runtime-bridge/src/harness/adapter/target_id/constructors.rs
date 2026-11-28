use super::BridgeHarnessTargetId;
use crate::facade::{TruthBranchIdentity, TruthCommitIdentity};
use crate::source::SourceDeclarationIdentity;
use crate::structural::StructuralIdentityDeclarationIdentity;

impl BridgeHarnessTargetId {
    pub fn committed_route(commit_identity: TruthCommitIdentity) -> Self {
        Self::CommittedRoute { commit_identity }
    }

    pub fn stream_routing(commit_window: impl IntoIterator<Item = TruthCommitIdentity>) -> Self {
        Self::StreamRouting {
            commit_window: commit_window.into_iter().collect(),
        }
    }

    pub fn stream_replay_audit(
        commit_window: impl IntoIterator<Item = TruthCommitIdentity>,
    ) -> Self {
        Self::StreamReplayAudit {
            commit_window: commit_window.into_iter().collect(),
        }
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

    pub fn structural_remap_exact(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapExact {
            declaration_identity,
        }
    }

    pub fn structural_remap_ambiguous(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapAmbiguous {
            declaration_identity,
        }
    }

    pub fn structural_remap_no_safe_match(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapNoSafeMatch {
            declaration_identity,
        }
    }

    pub fn structural_remap_lineage_divergence(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapLineageDivergence {
            declaration_identity,
        }
    }

    pub fn structural_remap_identity_conflict(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapIdentityConflict {
            declaration_identity,
        }
    }

    pub fn structural_remap_replay(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralRemapReplay {
            declaration_identity,
        }
    }

    pub fn structural_branch_compare(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralBranchCompare {
            declaration_identity,
        }
    }

    pub fn structural_branch_replay(
        declaration_identity: StructuralIdentityDeclarationIdentity,
    ) -> Self {
        Self::StructuralBranchReplay {
            declaration_identity,
        }
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

    pub fn writeback_duplicate_certification() -> Self {
        Self::WritebackDuplicateCertification
    }

    pub fn writeback_authority_denial_certification() -> Self {
        Self::WritebackAuthorityDenialCertification
    }

    pub fn writeback_feedback_loop_certification() -> Self {
        Self::WritebackFeedbackLoopCertification
    }

    pub fn writeback_replay_mismatch_certification() -> Self {
        Self::WritebackReplayMismatchCertification
    }

    pub fn writeback_extensible_family_certification() -> Self {
        Self::WritebackExtensibleFamilyCertification
    }

    pub fn writeback_multi_family_admission_boundary_certification() -> Self {
        Self::WritebackMultiFamilyAdmissionBoundaryCertification
    }

    pub fn writeback_cross_family_replay_loop_isolation_certification() -> Self {
        Self::WritebackCrossFamilyReplayLoopIsolationCertification
    }

    pub fn writeback_host_mapper_parity_certification() -> Self {
        Self::WritebackHostMapperParityCertification
    }
}
