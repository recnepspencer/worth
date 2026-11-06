use crate::runtime::{WorthQueryBridgeMutationArtifactIdentity, WorthQueryMutationEvidenceDigest};
use worth_runtime_bridge::facade::BridgeMutationAuthorityBundle;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationCausalityEvidence {
    causality_digest: WorthQueryMutationEvidenceDigest,
    truth_trigger_digest: WorthQueryMutationEvidenceDigest,
    route_digest: WorthQueryMutationEvidenceDigest,
    evaluation_surface_digest: WorthQueryMutationEvidenceDigest,
    truth_view_digest: WorthQueryMutationEvidenceDigest,
}

impl WorthQueryMutationCausalityEvidence {
    pub(in crate::runtime) fn from_bridge(bundle: &BridgeMutationAuthorityBundle) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let causality = bundle.causality();
        Ok(Self {
            causality_digest: imported_artifact("causality-root", causality.causality_digest())?,
            truth_trigger_digest: imported_artifact(
                "causality-truth-trigger",
                causality.truth_trigger_digest(),
            )?,
            route_digest: imported_artifact("causality-route", causality.route_digest())?,
            evaluation_surface_digest: imported_artifact(
                "causality-evaluation-surface",
                causality.evaluation_surface_digest(),
            )?,
            truth_view_digest: imported_artifact(
                "causality-truth-view",
                causality.truth_view_digest(),
            )?,
        })
    }

    pub fn causality_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.causality_digest
    }

    pub fn truth_trigger_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.truth_trigger_digest
    }

    pub fn route_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.route_digest
    }

    pub fn evaluation_surface_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.evaluation_surface_digest
    }

    pub fn truth_view_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.truth_view_digest
    }
}

fn imported_artifact(
    role: &'static str,
    artifact: impl Into<String>,
) -> Result<WorthQueryMutationEvidenceDigest, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    let artifact = WorthQueryBridgeMutationArtifactIdentity::imported(role, artifact)?;
    Ok(WorthQueryMutationEvidenceDigest::source_identity(role, artifact.evidence_identity())?)
}
