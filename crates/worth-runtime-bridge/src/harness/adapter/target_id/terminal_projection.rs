use std::fmt;

use super::BridgeHarnessTargetId;
use crate::facade::TruthCommitIdentity;

impl BridgeHarnessTargetId {
    pub(crate) fn as_external_target(&self) -> String {
        match self {
            Self::CommittedRoute { commit_identity } => commit_identity.as_str().to_owned(),
            Self::StreamRouting { commit_window } => {
                format!("stream-routing:{}", commit_window_string(commit_window))
            }
            Self::StreamReplayAudit { commit_window } => {
                format!(
                    "stream-replay-audit:{}",
                    commit_window_string(commit_window)
                )
            }
            Self::SourceMaterialize {
                declaration_identity,
            } => format!("source-materialize:{}", declaration_identity.as_str()),
            Self::SourceMaterializeBatch {
                declaration_identity,
            } => format!("source-materialize-batch:{}", declaration_identity.as_str()),
            Self::SourceReplay {
                declaration_identity,
            } => format!("source-replay:{}", declaration_identity.as_str()),
            Self::SourceRejectUnregistered {
                declaration_identity,
            } => format!(
                "source-reject-unregistered:{}",
                declaration_identity.as_str()
            ),
            Self::SourceRejectOpenSnapshot {
                declaration_identity,
            } => format!(
                "source-reject-open-snapshot:{}",
                declaration_identity.as_str()
            ),
            Self::SourceRejectSnapshotDrift {
                declaration_identity,
            } => format!(
                "source-reject-snapshot-drift:{}",
                declaration_identity.as_str()
            ),
            Self::PolicyProvenanceCertification => "policy-provenance-certify".to_owned(),
            Self::PolicyRejectionCertification => "policy-rejection-certify".to_owned(),
            Self::PolicyAmbientLeakCertification => "policy-ambient-leak-certify".to_owned(),
            Self::SpeculationDiscardCertification => "speculation-discard-certify".to_owned(),
            Self::SpeculationPromotionCertification => "speculation-promotion-certify".to_owned(),
            Self::SpeculationChurnCertification => "speculation-churn-certify".to_owned(),
            Self::StructuralRemapExact {
                declaration_identity,
            } => format!("structural-remap-exact:{}", declaration_identity.as_str()),
            Self::StructuralRemapAmbiguous {
                declaration_identity,
            } => format!(
                "structural-remap-ambiguous:{}",
                declaration_identity.as_str()
            ),
            Self::StructuralRemapNoSafeMatch {
                declaration_identity,
            } => format!(
                "structural-remap-no-safe-match:{}",
                declaration_identity.as_str()
            ),
            Self::StructuralRemapLineageDivergence {
                declaration_identity,
            } => format!(
                "structural-remap-lineage-divergence:{}",
                declaration_identity.as_str()
            ),
            Self::StructuralRemapIdentityConflict {
                declaration_identity,
            } => format!(
                "structural-remap-identity-conflict:{}",
                declaration_identity.as_str()
            ),
            Self::StructuralRemapReplay {
                declaration_identity,
            } => format!("structural-remap-replay:{}", declaration_identity.as_str()),
            Self::StructuralBranchCompare {
                declaration_identity,
            } => format!(
                "structural-branch-compare:{}",
                declaration_identity.as_str()
            ),
            Self::StructuralBranchReplay {
                declaration_identity,
            } => format!("structural-branch-replay:{}", declaration_identity.as_str()),
            Self::WritebackDuplicateCertification => "writeback-duplicate-certify".to_owned(),
            Self::WritebackAuthorityDenialCertification => {
                "writeback-authority-denial-certify".to_owned()
            }
            Self::WritebackFeedbackLoopCertification => "writeback-feedback-certify".to_owned(),
            Self::WritebackReplayMismatchCertification => {
                "writeback-replay-mismatch-certify".to_owned()
            }
            Self::WritebackExtensibleFamilyCertification => {
                "writeback-family-extension-certify".to_owned()
            }
            Self::WritebackMultiFamilyAdmissionBoundaryCertification => {
                "writeback-family-admission-boundary-certify".to_owned()
            }
            Self::WritebackCrossFamilyReplayLoopIsolationCertification => {
                "writeback-family-replay-loop-isolation-certify".to_owned()
            }
            Self::WritebackHostMapperParityCertification => {
                "writeback-family-mapper-parity-certify".to_owned()
            }
            Self::HistoricalCommit {
                branch_identity,
                commit_identity,
            } => format!(
                "history-commit:{}:{}",
                branch_identity.as_str(),
                commit_identity.as_str()
            ),
            Self::BranchHead { branch_identity } => {
                format!("branch-head:{}", branch_identity.as_str())
            }
        }
    }
}

impl fmt::Display for BridgeHarnessTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.as_external_target())
    }
}

fn commit_window_string(commit_window: &[TruthCommitIdentity]) -> String {
    commit_window
        .iter()
        .map(|commit| commit.as_str())
        .collect::<Vec<_>>()
        .join(",")
}
