use std::fmt;

use super::BridgeHarnessTargetId;

impl BridgeHarnessTargetId {
    pub(crate) fn as_external_target(&self) -> String {
        match self {
            Self::CommittedRoute { commit_identity } => commit_identity.as_str().to_owned(),
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
