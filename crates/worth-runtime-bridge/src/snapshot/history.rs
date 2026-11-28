use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::diagnostics::BridgeHistoricalMaterializationPath;
use crate::identity::{BridgeIdentity, HistoricalEvaluationArtifactIdentityTag};

use super::{MaterializedTruthViewObservation, TruthSnapshotIdentity};

pub type LoweredHistoricalEvaluationArtifactIdentity =
    BridgeIdentity<HistoricalEvaluationArtifactIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredHistoricalEvaluationArtifact {
    artifact_identity: LoweredHistoricalEvaluationArtifactIdentity,
    declaration_identity: crate::snapshot::HistoricalEvaluationDeclarationIdentity,
    planned_packet_digest: Arc<str>,
    authority_digest: Arc<str>,
    branch_identity: crate::input::envelope::TruthBranchIdentity,
    commit_identity: Option<crate::input::envelope::TruthCommitIdentity>,
    snapshot_identity: TruthSnapshotIdentity,
    materialization_path: BridgeHistoricalMaterializationPath,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl LoweredHistoricalEvaluationArtifact {
    pub(crate) fn lower(
        observation: &MaterializedTruthViewObservation,
        materialization_path: BridgeHistoricalMaterializationPath,
    ) -> Self {
        let authority_basis = observation.authority_basis();
        let commit_identity = authority_basis.commit_identity().cloned();
        let commit_identity_basis = commit_identity
            .as_ref()
            .map(|identity| format!("present:{}", identity.canonical_label()))
            .unwrap_or_else(|| "absent".to_string());
        let canonical_basis = Arc::<str>::from(format!(
            "lowered-historical-evaluation-artifact|declaration={}|planned={}|authority={}|branch={}|commit={}|snapshot={}|path={materialization_path:?}",
            observation.planned().declaration().declaration_identity().as_str(),
            observation.planned().digest(),
            authority_basis.digest(),
            authority_basis.branch_identity().canonical_label(),
            commit_identity_basis.as_str(),
            observation.snapshot_identity().canonical_label(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            artifact_identity: LoweredHistoricalEvaluationArtifactIdentity::admit_bridge_owned(
                format!("lowered-historical-evaluation-artifact:sha256:{digest:x}"),
            ),
            declaration_identity: observation
                .planned()
                .declaration()
                .declaration_identity()
                .clone(),
            planned_packet_digest: Arc::from(observation.planned().digest()),
            authority_digest: Arc::from(authority_basis.digest()),
            branch_identity: authority_basis.branch_identity().clone(),
            commit_identity,
            snapshot_identity: observation.snapshot_identity().clone(),
            materialization_path,
            canonical_basis,
            digest: Arc::from(format!(
                "lowered-historical-evaluation-artifact:sha256:{digest:x}"
            )),
        }
    }

    pub fn artifact_identity(&self) -> &LoweredHistoricalEvaluationArtifactIdentity {
        &self.artifact_identity
    }

    pub fn declaration_identity(
        &self,
    ) -> &crate::snapshot::HistoricalEvaluationDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn planned_packet_digest(&self) -> &str {
        self.planned_packet_digest.as_ref()
    }

    pub fn authority_digest(&self) -> &str {
        self.authority_digest.as_ref()
    }

    pub fn branch_identity(&self) -> &crate::input::envelope::TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn commit_identity(&self) -> Option<&crate::input::envelope::TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn materialization_path(&self) -> BridgeHistoricalMaterializationPath {
        self.materialization_path
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
